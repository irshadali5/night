//! 🌙 System Performance & Hardware Resource Monitor
//! Zero-dependency Linux procfs & sysfs reader for CPU, RAM, Disk, and Temperature metrics.
//! Designed for high-frequency status bar streaming with sub-millisecond execution.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

/// Real-time hardware performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemStats {
    pub cpu_pct: u8,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_pct: u8,
    pub cpu_temp_c: Option<u8>,
    pub disk_used_gb: u64,
    pub disk_total_gb: u64,
    pub disk_pct: u8,
}

impl SystemStats {
    /// CPU Nerd Font icon
    pub fn glyph_cpu(&self) -> &'static str {
        match self.cpu_pct {
            80..=100 => "󰓅",
            50..=79 => "",
            _ => "",
        }
    }

    /// Memory Nerd Font icon
    pub fn glyph_mem(&self) -> &'static str {
        match self.mem_pct {
            80..=100 => "󰍛",
            _ => "󰍛",
        }
    }

    /// Thermal Nerd Font icon
    pub fn glyph_temp(&self) -> &'static str {
        let temp = self.cpu_temp_c.unwrap_or(40);
        match temp {
            75..=255 => "󰔏",
            55..=74 => "󰔏",
            _ => "󰔏",
        }
    }

    /// Disk Nerd Font icon
    pub fn glyph_disk(&self) -> &'static str {
        "󰋊"
    }

    /// Formats CPU usage with icon (e.g. " 14%")
    pub fn display_cpu(&self) -> String {
        format!("{} {}%", self.glyph_cpu(), self.cpu_pct)
    }

    /// Formats RAM usage with icon (e.g. "󰍛 4.2/16.0 GiB (26%)")
    pub fn display_mem(&self) -> String {
        let used_gib = self.mem_used_mb as f32 / 1024.0;
        let total_gib = self.mem_total_mb as f32 / 1024.0;
        format!(
            "{} {:.1}/{:.1} GiB ({}%)",
            self.glyph_mem(),
            used_gib,
            total_gib,
            self.mem_pct
        )
    }

    /// Formats temperature with icon (e.g. "󰔏 45°C")
    pub fn display_temp(&self) -> String {
        match self.cpu_temp_c {
            Some(t) => format!("{} {}°C", self.glyph_temp(), t),
            None => format!("{} N/A", self.glyph_temp()),
        }
    }

    /// Formats root disk usage (e.g. "󰋊 120/480 GiB (25%)")
    pub fn display_disk(&self) -> String {
        format!(
            "{} {}/{} GiB ({}%)",
            self.glyph_disk(),
            self.disk_used_gb,
            self.disk_total_gb,
            self.disk_pct
        )
    }
}

/// Stores previous CPU idle and total ticks for delta calculation
static PREV_CPU: OnceLock<Mutex<(u64, u64)>> = OnceLock::new();

/// Samples current CPU usage percentage from `/proc/stat`
fn sample_cpu_pct() -> u8 {
    let stat = match fs::read_to_string("/proc/stat") {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let first_line = match stat.lines().next() {
        Some(l) if l.starts_with("cpu ") => l,
        _ => return 0,
    };

    let parts: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    if parts.len() < 4 {
        return 0;
    }

    let user = parts[0];
    let nice = parts[1];
    let system = parts[2];
    let idle = parts[3];
    let iowait = parts.get(4).copied().unwrap_or(0);
    let irq = parts.get(5).copied().unwrap_or(0);
    let softirq = parts.get(6).copied().unwrap_or(0);
    let steal = parts.get(7).copied().unwrap_or(0);

    let idle_time = idle + iowait;
    let non_idle_time = user + nice + system + irq + softirq + steal;
    let total_time = idle_time + non_idle_time;

    let cell = PREV_CPU.get_or_init(|| Mutex::new((idle_time, total_time)));
    let mut guard = cell.lock().unwrap();
    let (prev_idle, prev_total) = *guard;

    let total_delta = total_time.saturating_sub(prev_total);
    let idle_diff = idle_time.saturating_sub(prev_idle);

    *guard = (idle_time, total_time);

    if total_delta == 0 {
        return 0;
    }

    let usage = (total_delta.saturating_sub(idle_diff) as f64 / total_delta as f64) * 100.0;
    (usage.clamp(0.0, 100.0) as u8).min(100)
}

/// Reads RAM total and used megabytes from `/proc/meminfo`
fn read_mem_mb() -> (u64, u64, u8) {
    let meminfo = match fs::read_to_string("/proc/meminfo") {
        Ok(m) => m,
        Err(_) => return (0, 0, 0),
    };

    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            avail_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
        if total_kb > 0 && avail_kb > 0 {
            break;
        }
    }

    if total_kb == 0 {
        return (0, 0, 0);
    }

    let used_kb = total_kb.saturating_sub(avail_kb);
    let total_mb = total_kb / 1024;
    let used_mb = used_kb / 1024;
    let pct = ((used_kb as f64 / total_kb as f64) * 100.0) as u8;

    (used_mb, total_mb, pct.min(100))
}

/// Reads CPU temperature in Celsius from `/sys/class/thermal` or hwmon
fn read_cpu_temp() -> Option<u8> {
    // 1. Try /sys/class/thermal/thermal_zone0/temp
    let tz0 = Path::new("/sys/class/thermal/thermal_zone0/temp");
    if let Ok(content) = fs::read_to_string(tz0) {
        if let Ok(raw_millideg) = content.trim().parse::<i64>() {
            let deg = (raw_millideg / 1000).clamp(0, 150) as u8;
            return Some(deg);
        }
    }

    // 2. Try hwmon coretemp
    let hwmon_dir = Path::new("/sys/class/hwmon");
    if let Ok(entries) = fs::read_dir(hwmon_dir) {
        for entry in entries.flatten() {
            let temp_path = entry.path().join("temp1_input");
            if let Ok(content) = fs::read_to_string(&temp_path) {
                if let Ok(raw) = content.trim().parse::<i64>() {
                    let deg = (raw / 1000).clamp(0, 150) as u8;
                    return Some(deg);
                }
            }
        }
    }

    None
}

/// Reads root filesystem `/` disk metrics (used GB, total GB, %)
fn read_disk_gb() -> (u64, u64, u8) {
    let output = match Command::new("df").args(["-k", "/"]).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => return (0, 0, 0),
    };

    let mut lines = output.lines();
    let _header = lines.next();
    if let Some(row) = lines.next() {
        let cols: Vec<&str> = row.split_whitespace().collect();
        if cols.len() >= 4 {
            let total_kb = cols[1].parse::<u64>().unwrap_or(0);
            let used_kb = cols[2].parse::<u64>().unwrap_or(0);

            if total_kb > 0 {
                let total_gb = total_kb / (1024 * 1024);
                let used_gb = used_kb / (1024 * 1024);
                let pct = ((used_kb as f64 / total_kb as f64) * 100.0) as u8;
                return (used_gb, total_gb, pct.min(100));
            }
        }
    }

    (0, 0, 0)
}

/// Gathers comprehensive real-time system performance statistics
pub fn read_system_stats() -> SystemStats {
    let cpu_pct = sample_cpu_pct();
    let (mem_used_mb, mem_total_mb, mem_pct) = read_mem_mb();
    let cpu_temp_c = read_cpu_temp();
    let (disk_used_gb, disk_total_gb, disk_pct) = read_disk_gb();

    SystemStats {
        cpu_pct,
        mem_used_mb,
        mem_total_mb,
        mem_pct,
        cpu_temp_c,
        disk_used_gb,
        disk_total_gb,
        disk_pct,
    }
}
