#[allow(unused_imports)]
use crate::errors::RetiscopeError;

use directories::ProjectDirs;
use gpui::Global;
use std::fs;
use std::path::PathBuf;

/// Global access to the application's standard directory structure.
///
/// This struct follows the XDG Base Directory Specification on Linux,
/// Known Folders on Windows, and Standard Directories on macOS.
pub struct AppPaths {
    /// Directory for configuration files (e.g., `config.toml`).
    pub config: PathBuf,
    /// Directory for persistent data files (e.g., `identity.key`, databases).
    pub data: PathBuf,
    /// Directory for non-essential data (e.g., logs, temporary downloads).
    pub cache: PathBuf,
}

// GPUI Global
impl Global for AppPaths {}

impl AppPaths {
    /// Initialize paths and ensure directories exist.
    pub fn init() -> Self {
        let proj_dirs = ProjectDirs::from("org", "reticulum", "retiscope")
            .expect("Could not determine home directory");

        let paths = Self {
            config: proj_dirs.config_dir().to_path_buf(),
            data: proj_dirs.data_dir().to_path_buf(),
            cache: proj_dirs.cache_dir().to_path_buf(),
        };

        // Standard logic for ensuring folders exist
        let _ = fs::create_dir_all(&paths.config);
        let _ = fs::create_dir_all(&paths.data);
        let _ = fs::create_dir_all(&paths.cache);

        paths
    }

    /// Helper to get a path to a specific config file
    pub fn config_file(&self, name: &str) -> PathBuf {
        self.config.join(name)
    }

    pub fn get_connections_path(&self) -> PathBuf {
        self.config_file("connections.toml")
    }
    pub fn get_database_config_path(&self) -> PathBuf {
        self.config_file("database.toml")
    }
}

pub fn load_config_with_default<T>(path: PathBuf, default: T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or(default),
        Err(_) => {
            // file missing -> create the folder, save default
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&path, toml::to_string_pretty(&default).unwrap());
            default
        }
    }
}
