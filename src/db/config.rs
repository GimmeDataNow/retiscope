#[allow(unused_imports)]
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use crate::db::RetiscopeDB;
use crate::db::surrealdb;
use crate::errors::RetiscopeError;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration settings for the Retiscope database layer.
///
/// This struct holds the connection parameters and provider-specific options
/// required to initialize the database backend. It is designed to be deserialized
/// from configuration files (e.g., TOML).
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// Connection options for the selected database provider.
    pub database: DatabaseOptions,
    // Other general configuration options can be added here
}

/// Configuration options for different database backends.
///
/// This enum uses Serde's adjacency tagging to allow for different configuration
/// structures based on the `type` field in the configuration file.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DatabaseOptions {
    /// Configuration for a SurrealDB instance
    Surreal {
        /// The IP address or hostname of the SurrealDB server
        address: String,
        /// The network port the server is listening on
        port: u16,
        /// Whether to use Transport Layer Security (TLS) for the connection
        #[serde(default)]
        use_tls: bool,
        /// The namespace within SurrealDB
        namespace: String,
        /// The specific database name within the namespace
        database: String,
    },
    /// Configuration for a PostgreSQL instance
    /// (implementation is currently a `todo!` placeholder).
    Postgres { connection_string: String },
    /// Configuration for an IndexedDB instance
    /// (implementation is currently a `todo!` placeholder).
    IndexedDb { db_name: String },
}

impl Default for DatabaseConfig {
    /// Provides a default configuration pointing to a local SurrealDB instance.
    ///
    /// Defaults to:
    /// * Address: `127.0.0.1`
    /// * Port: `8000`
    /// * TLS: `false`
    /// * Namespace: `retiscope`
    /// * Database: `network`
    fn default() -> Self {
        Self {
            database: DatabaseOptions::Surreal {
                address: "127.0.0.1".to_string(),
                port: 8000,
                use_tls: false,
                namespace: "retiscope".to_string(),
                database: "network".to_string(),
            },
        }
    }
}

impl DatabaseConfig {
    /// Load the database configuration from a file on disk.
    ///
    /// The function reads the file at `path`, parses it as TOML, and
    /// returns the resulting `DatabaseConfig`.  If the file cannot be
    /// read or the TOML is invalid, the function logs a warning
    /// and falls back to `DatabaseConfig::default()`.
    #[instrument]
    pub fn load_database_config(path: PathBuf) -> DatabaseConfig {
        std::fs::read_to_string(&path)
            .and_then(|contents| {
                toml::from_str::<DatabaseConfig>(&contents)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                    .inspect_err(|_| error!("Failed to parse file"))
            })
            .unwrap_or_else(|_| {
                warn!("Failed to read file, using defaults");
                DatabaseConfig::default()
            })
    }

    /// Factory method to instantiate the concrete database implementation.
    ///
    /// Based on the variant held in `self.database`, this method initializes
    /// the appropriate driver and returns a thread-safe, reference-counted
    /// pointer to a [`RetiscopeDB`] implementation.
    ///
    /// # Errors
    ///
    /// This method will return an error if the underlying database driver
    /// fails to connect or if the provided configuration parameters are invalid.
    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub async fn create_db(&self) -> Result<Arc<dyn RetiscopeDB>, RetiscopeError> {
        match &self.database {
            DatabaseOptions::Surreal {
                address,
                port,
                use_tls,
                namespace,
                database,
            } => {
                let instance =
                    surrealdb::SurrealImpl::new(address, port, *use_tls, namespace, database)
                        .await?;
                return Ok(Arc::new(instance));
            }
            DatabaseOptions::Postgres { connection_string } => {
                todo!()
                // Ok(Arc::new(PostgresDbImpl::new(connection_string).await?))
            }
            DatabaseOptions::IndexedDb { db_name } => {
                todo!()
                // Ok(Arc::new(IndexedDbImpl::new(db_name).await?))
            }
        }
    }
}
