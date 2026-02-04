use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rivulet::app::AppContext;
use rivulet::cli::{commands, Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let ctx = AppContext::new(None)?;

    match cli.command {
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
            rivulet::tui::run(Arc::new(ctx)).await?;
        }
    }

    Ok(())
}
