use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub native_bind: String,
    pub http_bind: String,
    pub database_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "anthropic")]
    Anthropic {
        api_key: String,
        default_model: String,
    },
    #[serde(rename = "openai")]
    OpenAI {
        api_key: String,
        default_model: String,
    },
    #[serde(rename = "openai-compatible")]
    OpenAICompatible {
        endpoint: String,
        api_key: Option<String>,
        default_model: String,
    },
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .context(format!("Failed to read config from {:?}", config_path))?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    fn default_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".config/tty-server/config.toml"))
    }

    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "anthropic".to_string(),
            ProviderConfig::Anthropic {
                api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                default_model: "claude-sonnet-4-20250514".to_string(),
            },
        );

        Self {
            server: ServerConfig {
                native_bind: "127.0.0.1:7331".to_string(),
                http_bind: "127.0.0.1:7332".to_string(),
                database_path: Self::default_database_path()
                    .unwrap_or_else(|_| "./sessions.db".to_string()),
            },
            auth: AuthConfig {
                jwt_secret: "CHANGE_ME_IN_PRODUCTION".to_string(),
                token_expiry_hours: 24,
            },
            providers,
        }
    }

    fn default_database_path() -> Result<String> {
        let home = std::env::var("HOME")?;
        Ok(format!("{}/.local/share/tty-server/sessions.db", home))
    }
}
