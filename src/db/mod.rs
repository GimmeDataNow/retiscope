use crate::errors::RetiscopeError;
pub mod config;

pub mod surrealdb;
// pub mod postgres; // Future expansion

use crate::core::{AnnounceData, StoredAnnounce, StoredNode};

use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;

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
