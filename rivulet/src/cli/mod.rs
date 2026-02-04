pub mod commands;

use clap::{Parser, Subcommand};

use crate::fetcher::parallel::DEFAULT_WORKERS;

#[derive(Parser)]
#[command(name = "rivulet")]
#[command(about = "A terminal RSS/Atom reader", long_about = None)]
pub struct Cli {
    /// Number of parallel workers for fetching feeds
    #[arg(short, long, default_value_t = DEFAULT_WORKERS, global = true)]
    pub workers: usize,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new feed
    Add {
        /// URL of the feed to add
        url: String,
    },
    /// Remove a feed
    Remove {
        /// URL of the feed to remove
        url: String,
    },
    /// Import feeds from an OPML file
    Import {
        /// Path to the OPML file
        path: std::path::PathBuf,
    },
    /// Update all feeds
    Update,
    /// List feeds or items
    List {
        /// Show items instead of feeds
        #[arg(long)]
        items: bool,
    },
    /// Launch the TUI
    Tui,
    /// Background daemon for automatic updates
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the background daemon
    Start {
        /// Update interval (e.g., "1h", "30m", "6h", "1d")
        #[arg(short, long, default_value = "1h")]
        interval: String,

        /// Skip initial update on start
        #[arg(long)]
        no_initial_update: bool,

        /// Log file path (default: stdout)
        #[arg(short, long)]
        log: Option<std::path::PathBuf>,

        /// Run in foreground (don't detach)
        #[arg(short, long)]
        foreground: bool,
    },
    /// Stop the running daemon
    Stop,
    /// Check daemon status
    Status,
}
