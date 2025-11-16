use configparser::ini::Ini;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DebugConfig {
    provider: String,  // "echo|shell|lechat"
    log_level: String,
    log_file: String,
}

#[derive(Debug, Deserialize)]
struct LeChatConfig {
    api_key: String,
    endpoint: String,
    model: String,
    stream: bool,
    // ... other fields
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut conf = Ini::new();
        conf.load(path)?;

        let debug = conf.get_section("debug").unwrap();
        let debug_config = DebugConfig {
            provider: debug.get("provider").unwrap().to_string(),
            log_level: debug.get("log_level").unwrap().to_string(),
            log_file: debug.get("log_file").unwrap().to_string(),
        };

        let lechat = conf.get_section("provider.lechat").unwrap();
        let lechat_config = LeChatConfig {
            api_key: lechat.get("api_key").unwrap().to_string(),
            // ... parse other fields
        };

        Ok(Config { debug, lechat, .. })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match key {
            "debug.provider" => Some(&self.debug.provider),
            // ... other keys
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        match key {
            "debug.provider" => self.debug.provider = value.to_string(),
            // ... other keys
            _ => ()
        }
    }
}
