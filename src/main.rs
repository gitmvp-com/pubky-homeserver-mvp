use anyhow::Result;
use clap::Parser;
use pubky_homeserver_mvp::{Config, Server, Storage};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn default_data_dir_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".pubky-mvp")
}

fn validate_data_dir_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() && path.is_file() {
        return Err(format!("Given path is not a directory: {}", path.display()));
    }
    Ok(path)
}

#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Path to data directory. Defaults to ~/.pubky-mvp
    #[clap(short, long, default_value_os_t = default_data_dir_path(), value_parser = validate_data_dir_path)]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Create data directory if it doesn't exist
    if !args.data_dir.exists() {
        std::fs::create_dir_all(&args.data_dir)?;
        tracing::info!("Created data directory: {}", args.data_dir.display());
    }

    // Load or create config
    let config = Config::load_or_create(&args.data_dir)?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}={}", env!("CARGO_PKG_NAME").replace('-', "_"), config.logging.level)
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Using data directory: {}", args.data_dir.display());
    tracing::info!("Config loaded: {:?}", config);

    // Initialize storage
    let db_path = args.data_dir.join(&config.storage.db_path);
    std::fs::create_dir_all(&db_path)?;
    let storage = Storage::new(&db_path)?;
    tracing::info!("Storage initialized at: {}", db_path.display());

    // Start server
    let server = Server::new(config.clone(), storage);
    let listen_addr = config.server.listen_socket.clone();

    tracing::info!("Starting HTTP server on http://{}", listen_addr);
    tracing::info!("Press Ctrl+C to stop the server");

    server.run(&listen_addr).await?;

    Ok(())
}
