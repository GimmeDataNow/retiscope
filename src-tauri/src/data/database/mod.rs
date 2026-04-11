//! # Database Persistence & Configuration Layer
//!
//! This module provides a **provider‑agnostic** abstraction for all database
//! interactions in Retiscope.  It allows the application to stay completely
//! independent of the chosen backend – whether that is **SurrealDB** (the
//! current default), **PostgreSQL**, or an **IndexedDB** instance.
//!
//! The design is split into three logical parts:
//!
//! 1. **Configuration** – `DatabaseConfig` & `DatabaseOptions` describe a
//!    connection in a strongly‑typed way and can be deserialized from a
//!    TOML file.
//! 2. **Abstraction Layer** – the `RetiscopeDB` trait defines the public
//!    interface that all drivers must implement.  It covers schema setup,
//!    runtime initialisation, data persistence, and real‑time change streams.
//! 3. **Concrete Implementations** – each driver lives in its own sub‑module
//!    (currently only `surrealdb` is implemented).  New drivers can be added
//!    by implementing the trait and extending the `DatabaseOptions` enum.
//!
//! ## Configuration
//! The configuration is deserialized from TOML using Serde’s adjacency
//! tagging.  Example snippets for each supported provider follow.
//!
//! ### SurrealDB (default for development)
//! ```toml
//! [database]
//! type = "surreal"
//! address = "127.0.0.1"
//! port = 8000
//! use_tls = false
//! namespace = "retiscope"
//! database = "network"
//! ```
//!
//! ### PostgreSQL
//! ```toml
//! [database]
//! type = "postgres"
//! connection_string = "postgresql://user:pass@localhost/retiscope"
//! ```
//!
//! ### IndexedDB
//! ```toml
//! [database]
//! type = "indexeddb"
//! db_name = "retiscope"
//! ```
//!
//! The `DatabaseConfig::default()` implementation points to a local
//! SurrealDB instance and is meant only for quick local testing – do **not**
//! use it in production builds.
//!
//! ## Trait – `RetiscopeDB`
//! All drivers must implement this trait, which guarantees a uniform API across
//! providers.  The key responsibilities are:
//!
//! * **Schema bootstrap** – `set_up_db` creates tables and indexes,
//!   idempotently.
//! * **Runtime initialisation** – `init_db` authenticates and selects the
//!   correct namespace/database.
//! * **Persisting data** – `save_announces` performs an “upsert” for every
//!   `AnnounceData` and updates the related nodes’ timestamps.
//! * **Real‑time streams** – `watch_announces` / `watch_nodes` return
//!   `futures::channel::mpsc::UnboundedReceiver` streams that emit
//!   `StoredAnnounce`/`StoredNode` as they are committed to the database.
//!
//! ## Error handling
//! All trait methods return `Result<_, RetiscopeError>`.  For database‑level
//! errors (`sqlx::Error`, `surrealdb::Error`, etc.) those are wrapped
//! appropriately.  The configuration loader falls back to the default
//! configuration on parse errors and logs a warning.
//!
//! ## Extending the database layer
//! 1. Add a new variant to `DatabaseOptions` and implement any required
//!    fields.<br>
//! 2. Create a new sub‑module (e.g. `postgres.rs`) that implements
//!    `RetiscopeDB` for that provider.<br>
//! 3. Update `DatabaseConfig::create_db` to construct the new driver.
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

/// Sub‑module containing the SurrealDB implementation of
/// `RetiscopeDB`.  The implementation lives in
/// `surrealdb/mod.rs`.  New providers should be added in
/// analogous sub‑modules (e.g. `postgres.rs`, `indexeddb.rs`).
pub mod surrealdb;

/// # Retiscope Database Layer
///
/// The public API of this module is centered around the
/// [`RetiscopeDB`] trait.  Concrete drivers (currently only
/// SurrealDB) implement this trait so that the rest of the
/// application can stay completely agnostic to the underlying
/// database technology.
///
/// All trait methods are `async` and the trait itself is
/// `Send + Sync`, so a single instance can safely be shared
/// across threads (hence the `Arc<dyn RetiscopeDB>` returned by
/// `DatabaseConfig::create_db`).
#[async_trait]
pub trait RetiscopeDB: Send + Sync {
    /// Bootstrap the database schema and administrative users.
    ///
    /// The default implementation should be *idempotent* – it can be
    /// called repeatedly without causing errors.  It typically:
    ///   1. Creates the tables used by Retiscope.
    ///   2. Adds indexes that speed up queries.
    ///   3. Creates any internal system users that the driver
    ///      requires.
    ///
    /// This step is normally executed once when the application
    /// starts up.
    async fn set_up_db(&self) -> Result<(), RetiscopeError>;

    /// Authenticate and prepare the database for normal operation.
    ///
    /// The method must log in to the database, select the correct
    /// namespace/database (or schema), and perform any other
    /// runtime‑initialisation that the driver needs.
    async fn init_db(&self) -> Result<(), RetiscopeError>;

    /// Persist a collection of `AnnounceData` objects.
    ///
    /// Each `AnnounceData` is **upserted**: if an identical
    /// announcement already exists it is updated; otherwise it is
    /// inserted.  The method also updates the timestamps of the
    /// nodes referenced by the announces.
    async fn save_announces(&self, announce: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError>;

    /// Subscribe to real‑time updates of stored announcements.
    ///
    /// The returned `UnboundedReceiver<StoredAnnounce>` behaves
    /// like a stream that you can `recv().await` from.  Because
    /// it is *unbounded* there is no built‑in back‑pressure,
    /// so consumers should keep up with the stream or risk
    /// memory bloat.
    async fn watch_announces(&self) -> Result<UnboundedReceiver<StoredAnnounce>, RetiscopeError>;

    /// Subscribe to real‑time updates of stored node information.
    ///
    /// The returned `UnboundedReceiver<StoredAnnounce>` behaves
    /// like a stream that you can `recv().await` from.  Because
    /// it is *unbounded* there is no built‑in back‑pressure,
    /// so consumers should keep up with the stream or risk
    /// memory bloat.
    async fn watch_nodes(&self) -> Result<UnboundedReceiver<StoredNode>, RetiscopeError>;
}

/// Configuration settings for the Retiscope database layer.
///
/// This struct holds the connection parameters and provider-specific options
/// required to initialize the database backend. It is designed to be deserialized
/// from configuration files (e.g., TOML).
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

/// Load the database configuration from a file on disk.
///
/// The function reads the file at `path`, parses it as TOML, and
/// returns the resulting `DatabaseConfig`.  If the file cannot be
/// read or the TOML is invalid, the function logs a warning
/// and falls back to `DatabaseConfig::default()`.
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
