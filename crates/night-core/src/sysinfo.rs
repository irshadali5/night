//! 🌙 System Performance & Hardware Resource Monitor
//! Powered by the standard, actively-maintained `sysinfo` crate.
//! Provides robust, cross-platform CPU delta sampling, memory metrics,
//! thermal sensor monitoring, and filesystem statistics without manual procfs scraping or shell commands.

use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use sysinfo::{Components, Disks, System};

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

    /// Formats temperature with icon (e.g. "󰔏 52°C")
    pub fn display_temp(&self) -> String {
        if let Some(c) = self.cpu_temp_c {
            format!("{} {c}°C", self.glyph_temp())
        } else {
            "󰔏 N/A".to_string()
        }
    }

    /// Formats root disk usage with icon (e.g. "󰋊 120/480 GiB (25%)")
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

static SYSTEM_INSTANCE: OnceLock<Mutex<System>> = OnceLock::new();

fn get_system() -> &'static Mutex<System> {
    SYSTEM_INSTANCE.get_or_init(|| {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Mutex::new(sys)
    })
}

/// Gathers comprehensive real-time system performance statistics via `sysinfo`
pub fn read_system_stats() -> SystemStats {
    let (cpu_pct, mem_used_mb, mem_total_mb, mem_pct) = {
        let mut sys = crate::lock_unpoison(get_system());
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpu_pct = (sys.global_cpu_usage().round() as u8).min(100);

        let total_bytes = sys.total_memory();
        let used_bytes = sys.used_memory();
        let total_mb = total_bytes / (1024 * 1024);
        let used_mb = used_bytes / (1024 * 1024);
        let mem_pct = if total_bytes > 0 {
            ((used_bytes as f64 / total_bytes as f64) * 100.0) as u8
        } else {
            0
        }
        .min(100);

        (cpu_pct, used_mb, total_mb, mem_pct)
    };

    // Thermal sensors via Components
    let cpu_temp_c = {
        let components = Components::new_with_refreshed_list();
        components
            .iter()
            .filter_map(|c| c.temperature())
            .map(|t| (t.round() as u8).clamp(0, 150))
            .next()
    };

    // Filesystem metrics via Disks
    let (disk_used_gb, disk_total_gb, disk_pct) = {
        let disks = Disks::new_with_refreshed_list();
        let target_disk = disks
            .iter()
            .find(|d| d.mount_point() == std::path::Path::new("/") && d.total_space() > 0)
            .or_else(|| disks.iter().filter(|d| d.total_space() > 0).max_by_key(|d| d.total_space()));

        if let Some(disk) = target_disk {
            let total_b = disk.total_space();
            let avail_b = disk.available_space();
            let used_b = total_b.saturating_sub(avail_b);

            let total_gb = total_b / (1024 * 1024 * 1024);
            let used_gb = used_b / (1024 * 1024 * 1024);
            let pct = if total_b > 0 {
                ((used_b as f64 / total_b as f64) * 100.0) as u8
            } else {
                0
            }
            .min(100);

            (used_gb, total_gb, pct)
        } else {
            (0, 0, 0)
        }
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_stats() {
        let stats = read_system_stats();
        println!("CPU: {}%", stats.cpu_pct);
        println!("Mem: {}/{} MB ({}%)", stats.mem_used_mb, stats.mem_total_mb, stats.mem_pct);
        println!("Disk: {}/{} GB ({}%)", stats.disk_used_gb, stats.disk_total_gb, stats.disk_pct);
        println!("Temp: {:?}", stats.cpu_temp_c);
        let disks = Disks::new_with_refreshed_list();
        println!("Disks found: {}", disks.len());
        for d in &disks {
            println!("  mount: {:?}, total: {}, avail: {}", d.mount_point(), d.total_space(), d.available_space());
        }
    }
}
