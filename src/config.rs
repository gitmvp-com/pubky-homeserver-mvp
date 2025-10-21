use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_socket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                server_name: "pubky-homeserver-mvp".to_string(),
            },
            server: ServerConfig {
                listen_socket: "127.0.0.1:8080".to_string(),
            },
            storage: StorageConfig {
                db_path: "data/lmdb".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        }
    }
}

impl Config {
    /// Load config from data directory or create default if it doesn't exist
    pub fn load_or_create(data_dir: &Path) -> Result<Self> {
        let config_path = data_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
            Ok(config)
        } else {
            let config = Config::default();
            let content = toml::to_string_pretty(&config)
                .context("Failed to serialize default config")?;
            std::fs::write(&config_path, content)
                .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
            tracing::info!("Created default config file: {}", config_path.display());
            Ok(config)
        }
    }
}
