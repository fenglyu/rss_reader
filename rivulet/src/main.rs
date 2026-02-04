use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rivulet::app::AppContext;
use rivulet::cli::{commands, Cli, Commands, DaemonAction};
use rivulet::config::Config;
use rivulet::daemon::{Daemon, DaemonConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Load config for scraper settings
    let config = Config::load().unwrap_or_else(|e| {
        tracing::debug!("Failed to load config: {}. Using defaults.", e);
        Config::default()
    });

    // Create app context with scraper enabled based on config
    let ctx = AppContext::with_scraper_config(None, cli.workers, Some(config.scraper.clone()))?;

    match cli.command {
        Commands::Init { force } => {
            commands::init_config(force)?;
            return Ok(());
        }
        Commands::Add { url } => {
            commands::add_feed(&ctx, &url).await?;
        }
        Commands::Remove { url } => {
            commands::remove_feed(&ctx, &url).await?;
        }
        Commands::Import { path } => {
            commands::import_opml(&ctx, &path).await?;
        }
        Commands::Update => {
            commands::update_feeds(&ctx).await?;
        }
        Commands::List { items } => {
            if items {
                commands::list_items(&ctx)?;
            } else {
                commands::list_feeds(&ctx)?;
            }
        }
        Commands::Tui => {
            rivulet::tui::run(Arc::new(ctx), Arc::new(config)).await?;
        }
        Commands::Scrape {
            feed,
            limit,
            concurrency,
            visible,
        } => {
            commands::scrape_content(&ctx, feed.as_deref(), limit, concurrency, visible).await?;
        }
        Commands::Daemon { action } => {
            match action {
                DaemonAction::Start {
                    interval,
                    no_initial_update,
                    log,
                    foreground,
                } => {
                    let interval_secs = DaemonConfig::parse_interval(&interval)
                        .map_err(|e| anyhow::anyhow!(e))?;

                    let daemon_config = DaemonConfig {
                        update_interval_secs: interval_secs,
                        update_on_start: !no_initial_update,
                        log_file: log.clone(),
                    };

                    if foreground {
                        // Run in foreground
                        let daemon = Daemon::new(Arc::new(ctx), daemon_config);
                        daemon.run().await?;
                    } else {
                        // Detach and run in background
                        #[cfg(unix)]
                        {
                            use std::process::Command;

                            let mut args = vec![
                                "daemon".to_string(),
                                "start".to_string(),
                                "--foreground".to_string(),
                                "--interval".to_string(),
                                interval,
                            ];

                            if no_initial_update {
                                args.push("--no-initial-update".to_string());
                            }

                            if let Some(log_path) = log {
                                args.push("--log".to_string());
                                args.push(log_path.to_string_lossy().to_string());
                            }

                            let exe = std::env::current_exe()?;
                            Command::new(&exe)
                                .args(&args)
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .spawn()?;

                            println!("Daemon started in background");
                            println!("Use 'rivulet daemon status' to check status");
                            println!("Use 'rivulet daemon stop' to stop");
                        }

                        #[cfg(windows)]
                        {
                            use std::process::Command;
                            use std::os::windows::process::CommandExt;

                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                            const DETACHED_PROCESS: u32 = 0x00000008;

                            let mut args = vec![
                                "daemon".to_string(),
                                "start".to_string(),
                                "--foreground".to_string(),
                                "--interval".to_string(),
                                interval,
                            ];

                            if no_initial_update {
                                args.push("--no-initial-update".to_string());
                            }

                            if let Some(log_path) = log {
                                args.push("--log".to_string());
                                args.push(log_path.to_string_lossy().to_string());
                            }

                            let exe = std::env::current_exe()?;
                            Command::new(&exe)
                                .args(&args)
                                .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                                .spawn()?;

                            println!("Daemon started in background");
                            println!("Use 'rivulet daemon status' to check status");
                            println!("Use 'rivulet daemon stop' to stop");
                        }
                    }
                }
                DaemonAction::Stop => {
                    match rivulet::daemon::stop_daemon() {
                        Ok(()) => println!("Daemon stopped"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                DaemonAction::Status => {
                    println!("{}", rivulet::daemon::daemon_status());
                }
            }
        }
    }

    Ok(())
}
