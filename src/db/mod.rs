#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

pub mod config;
pub mod surrealdb;
// pub mod postgres; // Future expansion

use crate::core::{AnnounceData, StoredAnnounce, StoredNode};
use crate::db::config::DatabaseConfig;
use crate::errors::RetiscopeError;
use crate::paths::AppPaths;

use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;

use arc_swap::ArcSwap;
use std::sync::Arc;

#[allow(dead_code)]
#[async_trait]
pub trait RetiscopeDB: Send + Sync {
    async fn set_up_db(&self) -> Result<(), RetiscopeError>;
    async fn init_db(&self) -> Result<(), RetiscopeError>;
    async fn save_announces(&self, announce: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError>;
    async fn fetch_announces(&self) -> Result<Vec<StoredAnnounce>, RetiscopeError>;
    async fn fetch_nodes(&self) -> Result<Vec<StoredNode>, RetiscopeError>;
    async fn watch_announces(&self) -> Result<UnboundedReceiver<StoredAnnounce>, RetiscopeError>;
    async fn watch_nodes(&self) -> Result<UnboundedReceiver<StoredNode>, RetiscopeError>;
}

pub type DynDB = Option<Box<dyn RetiscopeDB>>;

pub struct DatabaseHandle {
    db: ArcSwap<DynDB>,
}

impl DatabaseHandle {
    pub fn new(db: DynDB) -> Self {
        Self {
            db: ArcSwap::from(Arc::new(db)),
        }
    }

    #[allow(dead_code)]
    /// Hot-swap the DB
    pub fn swap(&self, new_db: DynDB) {
        self.db.store(Arc::new(new_db));
    }

    /// Access the DB
    pub fn load(&self) -> Arc<DynDB> {
        self.db.load_full()
    }

    pub async fn create_and_configure() -> Self {
        let app_paths = AppPaths::init();
        let db_path = app_paths.get_database_config_path();

        // load, create, init
        let db = if let Some(config) = DatabaseConfig::load_database_config(db_path) {
            if let Ok(db_instance) = config.create_db().await {
                if db_instance.init_db().await.is_ok() {
                    Some(db_instance)
                } else {
                    error!("Failed to init DB");
                    None
                }
            } else {
                error!("Failed to create DB");
                None
            }
        } else {
            None
        };
        DatabaseHandle::new(db)
    }
}
