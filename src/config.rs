use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_application")]
    pub application: Application,
    #[serde(default = "default_ui")]
    pub ui: Ui,
    #[serde(default = "default_data")]
    pub data: Data,
    #[serde(default = "default_backups")]
    pub backups: Backups,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    #[serde(default = "default_currency")]
    pub default_currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ui {
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_percentage_threshold")]
    pub percentage_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    #[serde(default = "default_data_path")]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backups {
    #[serde(default = "default_backup_directory")]
    pub directory: String,
    #[serde(default = "default_backup_frequency")]
    pub frequency: String,
}

// Default value functions
fn default_application() -> Application {
    Application {
        default_currency: default_currency(),
    }
}

fn default_ui() -> Ui {
    Ui {
        date_format: default_date_format(),
        percentage_threshold: default_percentage_threshold(),
    }
}

fn default_data() -> Data {
    Data {
        path: default_data_path(),
    }
}

fn default_backups() -> Backups {
    Backups {
        directory: default_backup_directory(),
        frequency: default_backup_frequency(),
    }
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_date_format() -> String {
    "YYYY-MM-DD".to_string()
}

fn default_percentage_threshold() -> f64 {
    5.0
}

fn default_data_path() -> String {
    crate::db::default_db_path()
}

fn default_backup_directory() -> String {
    dirs::home_dir()
        .map(|p| {
            p.join(".dough")
                .join("backups")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "./backups".to_string())
}

fn default_backup_frequency() -> String {
    "monthly".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            application: default_application(),
            ui: default_ui(),
            data: default_data(),
            backups: default_backups(),
        }
    }
}

impl Config {
    /// Load config with priority order:
    /// 1. CLI path (if provided)
    /// 2. Environment variable DOUGH_CONFIG
    /// 3. Platform-specific user directories
    /// 4. Default values
    pub fn load(cli_path: Option<&str>) -> color_eyre::Result<Self> {
        // Try CLI path first
        if let Some(path) = cli_path {
            if let Ok(config) = Self::load_from_path(path) {
                return Ok(config);
            }
        }

        // Try environment variable
        if let Ok(env_path) = std::env::var("DOUGH_CONFIG") {
            if let Ok(config) = Self::load_from_path(&env_path) {
                return Ok(config);
            }
        }

        // Try platform-specific user directories
        for path in Self::config_search_paths() {
            if path.exists() {
                if let Ok(config) = Self::load_from_path(path.to_str().unwrap()) {
                    return Ok(config);
                }
            }
        }

        // Return defaults if no config found
        Ok(Self::default())
    }

    fn load_from_path(path: &str) -> color_eyre::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    fn config_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Platform-specific paths
        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                paths.push(
                    home.join("Library")
                        .join("Application Support")
                        .join("dough")
                        .join("config.toml"),
                );
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(data_dir) = dirs::data_dir() {
                paths.push(data_dir.join("dough").join("config.toml"));
            }
        }

        // XDG config path (Linux/Unix/Mac fallback)
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("dough").join("config.toml"));
        }

        // Home directory fallback
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".dough").join("config.toml"));
        }

        paths
    }
}

