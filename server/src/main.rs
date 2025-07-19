use fileshare_server::{config::AppConfig, run_server};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "fileshare-server")]
#[command(about = "A high-performance file sharing server")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Enable development mode
    #[arg(long)]
    dev: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("fileshare_server={},tower_http=debug", cli.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting rouillecloud server");
    
    // Load configuration
    let config = AppConfig::from_file(&cli.config).await?;
    
    if cli.dev {
        tracing::warn!("Running in development mode - this should not be used in production!");
    }
    
    // Start the server
    run_server(config).await?;
    
    Ok(())
}
