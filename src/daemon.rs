//! Background daemon for automatic feed updates.
//!
//! Provides Chrome-updater-style background updates without requiring
//! system scheduler configuration.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, Utc};
use tokio::time::interval;

use crate::app::AppContext;
use crate::store::Store;

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Update interval in seconds (default: 3600 = 1 hour)
    pub update_interval_secs: u64,
    /// Whether to run an update immediately on start
    pub update_on_start: bool,
    /// Log file path (None = stdout)
    pub log_file: Option<PathBuf>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            update_interval_secs: 3600, // 1 hour
            update_on_start: true,
            log_file: None,
        }
    }
}

impl DaemonConfig {
    /// Parse interval string like "1h", "30m", "6h", "1d"
    pub fn parse_interval(s: &str) -> Result<u64, String> {
        let s = s.trim().to_lowercase();

        if let Some(hours) = s.strip_suffix('h') {
            hours
                .parse::<u64>()
                .map(|h| h * 3600)
                .map_err(|_| format!("Invalid hours: {}", hours))
        } else if let Some(minutes) = s.strip_suffix('m') {
            minutes
                .parse::<u64>()
                .map(|m| m * 60)
                .map_err(|_| format!("Invalid minutes: {}", minutes))
        } else if let Some(days) = s.strip_suffix('d') {
            days.parse::<u64>()
                .map(|d| d * 86400)
                .map_err(|_| format!("Invalid days: {}", days))
        } else if let Some(secs) = s.strip_suffix('s') {
            secs.parse::<u64>()
                .map_err(|_| format!("Invalid seconds: {}", secs))
        } else {
            // Try parsing as raw seconds
            s.parse::<u64>()
                .map_err(|_| format!("Invalid interval: {}. Use format like '1h', '30m', '1d'", s))
        }
    }

    /// Format interval for display
    pub fn format_interval(secs: u64) -> String {
        if secs >= 86400 && secs.is_multiple_of(86400) {
            format!("{}d", secs / 86400)
        } else if secs >= 3600 && secs.is_multiple_of(3600) {
            format!("{}h", secs / 3600)
        } else if secs >= 60 && secs.is_multiple_of(60) {
            format!("{}m", secs / 60)
        } else {
            format!("{}s", secs)
        }
    }
}

/// Daemon runner
pub struct Daemon {
    ctx: Arc<AppContext>,
    config: DaemonConfig,
    running: Arc<AtomicBool>,
}

impl Daemon {
    pub fn new(ctx: Arc<AppContext>, config: DaemonConfig) -> Self {
        Self {
            ctx,
            config,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Get the PID file path
    pub fn pid_file_path() -> Option<PathBuf> {
        dirs::runtime_dir()
            .or_else(dirs::cache_dir)
            .map(|d| d.join("rivulet").join("daemon.pid"))
    }

    /// Check if another daemon is already running
    pub fn is_running() -> bool {
        if let Some(pid_path) = Self::pid_file_path() {
            if pid_path.exists() {
                if let Ok(pid_str) = fs::read_to_string(&pid_path) {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        // Check if process is still running
                        return Self::process_exists(pid);
                    }
                }
            }
        }
        false
    }

    #[cfg(unix)]
    fn process_exists(pid: u32) -> bool {
        use std::process::Command;
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    fn process_exists(pid: u32) -> bool {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }

    /// Write PID file
    fn write_pid_file(&self) -> std::io::Result<()> {
        if let Some(pid_path) = Self::pid_file_path() {
            if let Some(parent) = pid_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = fs::File::create(&pid_path)?;
            writeln!(file, "{}", std::process::id())?;
        }
        Ok(())
    }

    /// Remove PID file
    fn remove_pid_file(&self) {
        if let Some(pid_path) = Self::pid_file_path() {
            let _ = fs::remove_file(pid_path);
        }
    }

    /// Log a message with timestamp
    fn log(&self, msg: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] {}", timestamp, msg);

        if let Some(ref log_path) = self.config.log_file {
            if let Ok(mut file) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let _ = writeln!(file, "{}", line);
            }
        } else {
            println!("{}", line);
        }
    }

    /// Run the daemon
    pub async fn run(&self) -> crate::app::Result<()> {
        // Check for existing daemon
        if Self::is_running() {
            return Err(crate::app::RivuletError::Other(
                "Another daemon instance is already running".to_string(),
            ));
        }

        // Write PID file
        self.write_pid_file().map_err(|e| {
            crate::app::RivuletError::Other(format!("Failed to write PID file: {}", e))
        })?;

        // Set up signal handler for graceful shutdown
        let running = self.running.clone();

        #[cfg(unix)]
        {
            let running_clone = running.clone();
            tokio::spawn(async move {
                let mut sigterm =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("Failed to set up SIGTERM handler");
                let mut sigint =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                        .expect("Failed to set up SIGINT handler");

                tokio::select! {
                    _ = sigterm.recv() => {},
                    _ = sigint.recv() => {},
                }
                running_clone.store(false, Ordering::SeqCst);
            });
        }

        #[cfg(windows)]
        {
            let running_clone = running.clone();
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                running_clone.store(false, Ordering::SeqCst);
            });
        }

        self.log(&format!(
            "Rivulet daemon started (update interval: {}, PID: {})",
            DaemonConfig::format_interval(self.config.update_interval_secs),
            std::process::id()
        ));

        // Run initial update if configured
        if self.config.update_on_start {
            self.log("Running initial update...");
            self.run_update().await;
        }

        // Main loop
        let mut timer = interval(Duration::from_secs(self.config.update_interval_secs));
        timer.tick().await; // Skip the first immediate tick

        while self.running.load(Ordering::SeqCst) {
            timer.tick().await;

            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            self.log("Running scheduled update...");
            self.run_update().await;
        }

        self.log("Daemon shutting down...");
        self.remove_pid_file();

        Ok(())
    }

    /// Run a single update cycle
    async fn run_update(&self) {
        let start = Utc::now();

        match self.ctx.store.get_all_feeds() {
            Ok(feeds) => {
                if feeds.is_empty() {
                    self.log("No feeds to update");
                    return;
                }

                let results = self
                    .ctx
                    .parallel_fetcher
                    .fetch_all(feeds, self.ctx.store.clone(), &self.ctx.normalizer)
                    .await;

                let mut total_new = 0;
                let mut errors = 0;

                for (feed_id, result) in results {
                    match result {
                        Ok(count) => {
                            total_new += count;
                            if count > 0 {
                                if let Ok(Some(feed)) = self.ctx.store.get_feed(feed_id) {
                                    self.log(&format!(
                                        "  {} new items from {}",
                                        count,
                                        feed.display_title()
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            errors += 1;
                            if let Ok(Some(feed)) = self.ctx.store.get_feed(feed_id) {
                                self.log(&format!(
                                    "  Error updating {}: {}",
                                    feed.display_title(),
                                    e
                                ));
                            }
                        }
                    }
                }

                let elapsed = Utc::now().signed_duration_since(start);
                self.log(&format!(
                    "Update complete: {} new items, {} errors ({:.1}s)",
                    total_new,
                    errors,
                    elapsed.num_milliseconds() as f64 / 1000.0
                ));
            }
            Err(e) => {
                self.log(&format!("Failed to get feeds: {}", e));
            }
        }
    }

    /// Stop the daemon (called externally)
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Stop a running daemon by reading PID file and sending signal
pub fn stop_daemon() -> Result<(), String> {
    let pid_path =
        Daemon::pid_file_path().ok_or_else(|| "Could not determine PID file path".to_string())?;

    if !pid_path.exists() {
        return Err("No daemon is running (PID file not found)".to_string());
    }

    let pid_str =
        fs::read_to_string(&pid_path).map_err(|e| format!("Failed to read PID file: {}", e))?;

    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| "Invalid PID in PID file".to_string())?;

    #[cfg(unix)]
    {
        use std::process::Command;
        let status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| format!("Failed to send signal: {}", e))?;

        if status.success() {
            // Remove PID file
            let _ = fs::remove_file(&pid_path);
            Ok(())
        } else {
            Err(format!("Failed to stop daemon (PID {})", pid))
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(|e| format!("Failed to stop process: {}", e))?;

        if status.success() {
            let _ = fs::remove_file(&pid_path);
            Ok(())
        } else {
            Err(format!("Failed to stop daemon (PID {})", pid))
        }
    }
}

/// Check daemon status
pub fn daemon_status() -> String {
    if let Some(pid_path) = Daemon::pid_file_path() {
        if pid_path.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_path) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if Daemon::process_exists(pid) {
                        return format!("Daemon is running (PID: {})", pid);
                    } else {
                        return "Daemon is not running (stale PID file)".to_string();
                    }
                }
            }
        }
    }
    "Daemon is not running".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval() {
        assert_eq!(DaemonConfig::parse_interval("1h").unwrap(), 3600);
        assert_eq!(DaemonConfig::parse_interval("30m").unwrap(), 1800);
        assert_eq!(DaemonConfig::parse_interval("1d").unwrap(), 86400);
        assert_eq!(DaemonConfig::parse_interval("60s").unwrap(), 60);
        assert_eq!(DaemonConfig::parse_interval("3600").unwrap(), 3600);
        assert_eq!(DaemonConfig::parse_interval("6h").unwrap(), 21600);
        assert!(DaemonConfig::parse_interval("invalid").is_err());
    }

    #[test]
    fn test_format_interval() {
        assert_eq!(DaemonConfig::format_interval(3600), "1h");
        assert_eq!(DaemonConfig::format_interval(1800), "30m");
        assert_eq!(DaemonConfig::format_interval(86400), "1d");
        assert_eq!(DaemonConfig::format_interval(90), "90s");
        assert_eq!(DaemonConfig::format_interval(7200), "2h");
    }
}
