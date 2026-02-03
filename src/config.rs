use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

// config options
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub editor_cmd: Option<String>,
    pub auto_sync: Option<bool>,
    pub theme: Option<Theme>,
    pub sort_mode: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Theme {
    pub accent: Option<String>,
    pub selection: Option<String>,
    pub header: Option<String>,
    pub dim: Option<String>,
    pub bold: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor_cmd: None,
            auto_sync: Some(false),
            theme: None,
            sort_mode: Some("Date".to_string()),
        }
    }
}

const DEFAULT_CONFIG: &str = r##"# Kiroku Configuration

# Optional: Command to open your text editor.
# Examples: "vim", "nano", "code --wait", "nvim"
# editor_cmd = "vim"

# Optional: Automatically sync with git when exiting the application.
# Default is false.
auto_sync = false

# Optional: Default sort mode for notes.
# Options: "Date", "Name", "Size"
# sort_mode = "Date"

# Optional: Custom Color Theme
# You can uncomment and customize these hex codes.
# [theme]
# accent = "#89dceb"    # Borders and main highlights
# selection = "#bb9af7" # Selected item in the list
# header = "#89b4fa"    # Markdown headers
# dim = "#6c7086"       # Footer and dim text
# bold = "#f38ba8"      # Bold text and heavy emphasis
"##;

// load config
pub fn load_config() -> Result<Config> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_path = home_dir.join(".config").join("kiroku").join("config.toml");

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, DEFAULT_CONFIG)?;
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

// save config
pub fn save_config(config: &Config) -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_path = home_dir.join(".config").join("kiroku").join("config.toml");

    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;

    Ok(())
}
