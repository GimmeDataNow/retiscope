//! The Database Persistence and Configuration Layer.
//!
//! This module provides the infrastructure for managing database connections and
//! configurations within Retiscope. It abstracts the underlying database
//! through a unified trait, allowing the application to remain agnostic of whether
//! it is communicating with SurrealDB, PostgreSQL or IndexedDB.
//!
//! # Architecture
//!
//! The module is built upon a **Provider-Agnostic Pattern**:
//!
//! 1. **Configuration Layer**: Uses a strongly-typed, tagged enum (`DatabaseOptions`)
//!    to define connection parameters for various backends, supporting seamless
//!    deserialization from TOML configuration files.
//!
//! 2. **Abstraction Layer**: The [`RetiscopeDB`] trait defines the interface for
//!    all database implementations, ensuring that features like batch-saving
//!    announces and real-time "watching" are consistent across all drivers.
//!
//! # Implementation Notes
//!
//! * **SurrealDB**: Currently the primary implementation, utilizing namespace
//!   and database separation for multi-tenant-style isolation.
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::data::{AnnounceData, StoredAnnounce, StoredNode};
use crate::errors::RetiscopeError;
pub mod surrealdb;

/// An asynchronous trait representing the database persistence layer for Retiscope.
///
/// This trait provides an abstraction for managing database schemas, initializing
/// connections, persisting announcement data, and subscribing to real-time
/// updates for announces and nodes.
#[async_trait]
pub trait RetiscopeDB: Send + Sync {
    /// Initializes the database schema and administrative users.
    ///
    /// This method ensures the database is in a ready state by:
    /// * Applying required table schemas and indexes.
    /// * Provisioning internal system users and permissions.
    ///
    /// This is typically called during a system bootstrap.
    async fn set_up_db(&self) -> Result<(), RetiscopeError>;

    /// Prepares the database connection for operational use.
    ///
    /// This method ensures the correct database state for subsequent queries by:
    /// * Authenticating/logging into the correct database user.
    /// * Selecting the appropriate namespace and database schema.
    async fn init_db(&self) -> Result<(), RetiscopeError>;

    /// Persists a collection of announcement data to the database.
    ///
    /// This method performs an "upsert" (update or insert) operation for:
    /// * Each individual [`AnnounceData`] entry.
    /// * The associated nodes and their respective timestamps.
    async fn save_announces(&self, announce: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError>;

    /// Returns a real-time stream of stored announcements.
    ///
    /// This method provides an asynchronous, unbounded receiver that yields [`StoredAnnounce`]
    /// objects as they are successfully committed to the database.
    async fn watch_announces(&self) -> Result<UnboundedReceiver<StoredAnnounce>, RetiscopeError>;

    /// Returns a real-time stream of stored node updates.
    ///
    /// This method provides an asynchronous, unbounded receiver that yields [`StoredNode`]
    /// objects whenever node information is updated in the database.
    async fn watch_nodes(&self) -> Result<UnboundedReceiver<StoredNode>, RetiscopeError>;
}

/// Configuration settings for the Retiscope database layer.
///
/// This struct holds the connection parameters and provider-specific options
/// required to initialize the database backend. It is designed to be deserialized
/// from configuration files (e.g., TOML).
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// The specific connection options for the chosen database provider.
    pub database: DatabaseOptions,
    // Other settings...
    // pub log_level: String,
}

/// Configuration options for different database backends.
///
/// This enum uses Serde's adjacency tagging to allow for different configuration
/// structures based on the `type` field in the configuration file.
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
    Postgres { connection_string: String },
    /// Configuration for an IndexedDB instance
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

/// Loads the database configuration from a file on disk.
///
/// This function attempts to read a file at the provided path and parse it as TOML.
/// If the file cannot be read or the content is invalid, it will log a warning
/// and fall back to the [`DatabaseConfig::default()`] implementation.
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
