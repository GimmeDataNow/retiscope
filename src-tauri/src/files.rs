use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

/// A global, thread-safe singleton for application paths.
/// Initialized lazily on the first call to [`get_paths`].
pub static PATHS: OnceLock<AppPaths> = OnceLock::new();

/// Retrieves the global application paths, initializing them if necessary.
///
/// This function ensures that the `config`, `data`, and `cache` directories
/// exist on the filesystem before returning.
///
/// # Panics
/// Panics if the home directory cannot be determined or if the process lacks
/// permissions to create the required directories.
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

/// Helper to recursively create a directory if it does not already exist.
fn ensure_dir(path: &Path) {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .unwrap_or_else(|e| panic!("Failed to create directory at {:?}: {}", path, e));
    }
}

/// Atomically creates a new file at the specified path if it does not exist.
///
/// This is a "best-effort" creation and ignores errors (e.g., if the file already exists).
pub fn ensure_file<P>(path: P)
where
    P: AsRef<Path>,
{
    let _ = std::fs::File::create_new(path);
}

/// Persists the Reticulum identity key to the data directory.
///
/// On Unix-like systems, this function explicitly sets the file permissions
/// to `0o600` (read/write for the owner only) to protect the private key.
///
/// # Errors
/// Returns an [`std::io::Result`] if the file cannot be created, truncated, or if
/// permission settings fail.
pub fn save_identity(identity_data: &[u8]) -> std::io::Result<()> {
    let path = get_paths().data.join("identity.key");

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // override the prev file
        .open(&path)?;

    // set permissions to 600 (Unix only)
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

/// Configuration details for a remote Reticulum node connection.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteConnection {
    /// A human-readable identifier for the connection.
    pub alias: String,
    /// The destination address hash, represented as a hex string.
    pub address: String,
    /// Whether this connection is active for the current session.
    pub enabled: bool,
}

/// A wrapper for serializing a collection of [`RemoteConnection`]s,
/// primarily used for TOML configuration files.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteConnectionWrapper {
    pub connections: Vec<RemoteConnection>,
}

// #[tauri::command]
// fn load_connections() -> Result<Vec<RemoteConnection>, String> {
//     let path = get_paths().config.join("remote.toml");

//     if !path.exists() {
//         return Ok(vec![]);
//     }

//     let contents = std::fs::read_to_string(path)
//         .inspect_err(|e| error!(error = %e))
//         .map_err(|e| e.to_string())?;
//     let config: RemoteConnectionWrapper = toml::from_str(&contents)
//         .inspect_err(|e| error!(error = %e))
//         .map_err(|e| e.to_string())?;
//     Ok(config.connections)
// }
