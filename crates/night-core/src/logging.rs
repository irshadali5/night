//! 🌙 Production-Grade Logging Architecture for Project Night
//! Pure-Rust non-blocking rolling file logger using `tracing`, `tracing-subscriber`, and `tracing-appender`.
//! Automatically writes daily rotating logs to `~/.local/state/night/logs/<service>.log`
//! while preserving console output and capturing unhandled panics with full backtraces.

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Resolves standard XDG log directory: `$XDG_STATE_HOME/night/logs` or `~/.local/state/night/logs`
pub fn log_dir() -> PathBuf {
    let candidate = dirs::state_dir()
        .map(|d| d.join("night/logs"))
        .or_else(|| dirs::data_local_dir().map(|d| d.join("night/logs")))
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/state/night/logs")))
        .unwrap_or_else(|| PathBuf::from("/tmp/night/logs"));

    if fs::create_dir_all(&candidate).is_ok() && candidate.is_dir() {
        candidate
    } else {
        let fallback = PathBuf::from("/tmp/night/logs");
        let _ = fs::create_dir_all(&fallback);
        fallback
    }
}

/// Resolves current log file path for the specified service
pub fn log_file_path(service: &str) -> PathBuf {
    log_dir().join(format!("{service}.log"))
}

/// Initializes non-blocking structured logging for a service daemon or GUI client.
/// Returns a `WorkerGuard` which must be held in `main` to ensure logs flush on exit.
pub fn init_logger(service: &str) -> Option<WorkerGuard> {
    let dir = log_dir();
    let file_appender = tracing_appender::rolling::daily(&dir, format!("{service}.log"));
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    // Default filter allows informational messages from project night crates
    let default_filter = format!(
        "{service}=info,night_core=info,night_pinnacle=info,night_settings=info,night_binds=info,night_daemons=info,night_layout=info,pinnacle_api=info,info"
    );

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    // Structured file logger layer (uncolored, timestamped, formatted)
    let file_layer = fmt::layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(env_filter.clone());

    // Interactive terminal stderr layer (colored)
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(false)
        .with_filter(env_filter);

    // Combine subscriber layers
    let _ = tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .try_init();

    // Install global panic hook to guarantee all panics are captured in logs
    let service_name = service.to_string();
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unspecified panic payload".to_string()
        };

        let backtrace = std::backtrace::Backtrace::capture();

        error!(
            service = %service_name,
            location = %location,
            panic = %payload,
            backtrace = ?backtrace,
            "🔥 UNHANDLED PANIC DETECTED"
        );

        // Forward to standard panic hook as well
        default_panic_hook(panic_info);
    }));

    Some(guard)
}

/// Reads the last `line_count` lines from the service log file
pub fn read_recent_logs(service: &str, line_count: usize) -> Vec<String> {
    let dir = log_dir();
    let mut matching_files: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(service) && file_name.contains(".log") {
                    matching_files.push(path);
                }
            }
        }
    }

    matching_files.sort_by(|a, b| {
        let meta_a = a.metadata().and_then(|m| m.modified()).ok();
        let meta_b = b.metadata().and_then(|m| m.modified()).ok();
        meta_b.cmp(&meta_a)
    });

    let target_file = matching_files.into_iter().next().unwrap_or_else(|| log_file_path(service));

    if !target_file.exists() {
        return vec![format!("Log file not found: {}", target_file.display())];
    }

    let file = match File::open(&target_file) {
        Ok(f) => f,
        Err(e) => return vec![format!("Could not open {}: {e}", target_file.display())],
    };

    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        lines.push(line);
    }

    let total = lines.len();
    if total > line_count {
        lines.split_off(total - line_count)
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_paths() {
        let dir = log_dir();
        assert!(dir.is_dir() || dir.exists());
        let p = log_file_path("test-service");
        assert!(p.to_string_lossy().ends_with("test-service.log"));
    }

    #[test]
    fn test_read_recent_logs_nonexistent() {
        let res = read_recent_logs("nonexistent-test-service-12345", 10);
        assert!(!res.is_empty());
        assert!(res[0].contains("Log file not found"));
    }
}
