use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tracing::{debug, error, info, instrument, trace, warn};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Global access to the app's directories
#[allow(dead_code)]
pub struct AppPaths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
}

pub static PATHS: OnceLock<AppPaths> = OnceLock::new();

pub fn get_paths() -> &'static AppPaths {
    PATHS.get_or_init(|| {
        let proj_dirs = ProjectDirs::from("org", "reticulum", "retiscope")
            .expect("Could not determine a home directory for the current user");

        let paths = AppPaths {
            config: proj_dirs.config_dir().to_path_buf(),
            data: proj_dirs.data_dir().to_path_buf(),
            cache: proj_dirs.cache_dir().to_path_buf(),
        };

        // Ensure the directories exist immediately upon initialization
        ensure_dir(&paths.config);
        ensure_dir(&paths.data);
        ensure_dir(&paths.cache);

        paths
    })
}

fn ensure_dir(path: &Path) {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .unwrap_or_else(|e| panic!("Failed to create directory at {:?}: {}", path, e));
    }
}

pub fn ensure_file<P>(path: P)
where
    P: AsRef<Path>,
{
    let _ = std::fs::File::create_new(path);
}

pub fn save_identity(identity_data: &[u8]) -> std::io::Result<()> {
    let path = get_paths().data.join("identity.key");

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // override the prev file
        .open(&path)?;

    // set permissions to 600
    #[cfg(unix)]
    {
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms)?;
    }

    // write
    file.write_all(identity_data)?;
    Ok(())
}

/// Represents the reticulum connection that the front end needs to make
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteConnection {
    /// alias or name
    pub alias: String,
    /// address hash as hex string
    pub address: String,
    pub enabled: bool,
}

/// Wrapper type for use with TOML
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteConnectionWrapper {
    pub connections: Vec<RemoteConnection>,
}

#[tauri::command]
fn load_connections() -> Result<Vec<RemoteConnection>, String> {
    let path = get_paths().config.join("remote.toml");

    if !path.exists() {
        return Ok(vec![]);
    }

    let contents = std::fs::read_to_string(path)
        .inspect_err(|e| error!(error = %e))
        .map_err(|e| e.to_string())?;
    let config: RemoteConnectionWrapper = toml::from_str(&contents)
        .inspect_err(|e| error!(error = %e))
        .map_err(|e| e.to_string())?;
    Ok(config.connections)
}
