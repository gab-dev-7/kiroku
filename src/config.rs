use anyhow::Result;
use serde::Deserialize;
use std::fs;

// application configuration options
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub editor_cmd: Option<String>,
    #[allow(dead_code)]
    pub auto_sync: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor_cmd: None,
            auto_sync: Some(false),
        }
    }
}

// load configuration from standard location or return defaults
pub fn load_config() -> Result<Config> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_path = home_dir.join(".config").join("kiroku").join("config.toml");

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
