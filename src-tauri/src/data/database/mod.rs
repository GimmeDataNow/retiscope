use futures::channel::mpsc::UnboundedReceiver;
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use std::sync::Arc;

use crate::data::{AnnounceData, StoredAnnounce};
use crate::errors::RetiscopeError;

pub mod surrealdb;

#[async_trait]
pub trait RetiscopeDB: Send + Sync {
    /// Initializes the database schema and administrative users.
    ///
    /// This method ensures the database is in a ready state by:
    /// * Applying required table schemas and indexes.
    /// * Provisioning internal system users and permissions.
    ///
    /// # Errors
    ///
    /// TODO
    async fn set_up_db(&self) -> Result<(), RetiscopeError>;
    /// Prepares the database for use.
    ///
    /// This method ensures the correct database state by:
    /// * Logging into the correct user.
    /// * Selecting the right namespace and database.
    ///
    /// # Errors
    ///
    /// TODO
    async fn init_db(&self) -> Result<(), RetiscopeError>;

    /// Writes the announces to the database.
    ///
    /// This method writes this data by:
    /// * Upserting each announce
    /// * Upserting each node and the respective timestamps
    ///
    /// # Errors
    ///
    /// TODO
    async fn save_announces(&self, announce: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError>;

    async fn watch_announces(&self) -> Result<UnboundedReceiver<StoredAnnounce>, RetiscopeError>;
    async fn node_announces(&self) -> Result<(), RetiscopeError>;
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub database: DatabaseOptions,
    // Other settings...
    // pub log_level: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DatabaseOptions {
    Surreal {
        address: String,
        port: u16,
        #[serde(default)]
        use_tls: bool,
        namespace: String,
        database: String,
    },
    Postgres {
        connection_string: String,
    },
    IndexedDb {
        db_name: String,
    },
}

impl Default for DatabaseConfig {
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

#[instrument]
pub fn load_database_config(path: PathBuf) -> DatabaseConfig {
    fs::read_to_string(&path)
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
