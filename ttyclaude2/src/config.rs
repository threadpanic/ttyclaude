use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    pub vim_mode: bool,
    pub theme: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path()?;

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config from {:?}", config_path))?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(config_path, contents)?;

        Ok(())
    }

    fn default_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".config/ttyclaude2/config.toml"))
    }

    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "127.0.0.1:7331".to_string(),
            },
            auth: AuthConfig {
                token: "default_user".to_string(), // Will be replaced with JWT
            },
            ui: UiConfig {
                vim_mode: true,
                theme: "monokai".to_string(),
            },
        }
    }
}
