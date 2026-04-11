//! SurrealDB backend implementation for Retiscope.
//!
//! This module implements the [`RetiscopeDB`] trait using the SurrealDB `any` engine,
//! allowing for connections via WebSocket (WS/WSS), HTTP, or local file storage.
//!
//! # Schema
//! The implementation handles two main entities:
//! * `node`: Tracks unique network participants and their last-seen heartbeats.
//! * `announce`: A log of network announcements.
//!
//! # Performance
//! The [`SurrealImpl::save_announces`] method uses a batched UPSERT logic to minimize
//! round-trips to the database, ensuring efficient ingestion of high-frequency data.
use futures::{channel::mpsc, StreamExt};
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::data::{database::RetiscopeDB, AnnounceData, StoredAnnounce, StoredNode};
use crate::errors::RetiscopeError;

use async_trait::async_trait;

use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;

#[derive(Debug)]
#[cfg(feature = "surrealdb")]
pub struct SurrealImpl {
    pub connection: surrealdb::Surreal<Any>,
}

#[cfg(feature = "surrealdb")]
#[async_trait]
impl RetiscopeDB for SurrealImpl {
    #[instrument(skip(self))]
    async fn set_up_db(&self) -> Result<(), RetiscopeError> {
        todo!()
    }
    #[instrument(skip(self))]
    async fn init_db(&self) -> Result<(), RetiscopeError> {
        // sign in
        self.connection
            .signin(Root {
                username: "a".into(),
                password: "a".into(),
            })
            .await
            .inspect_err(|e| error!(error = %e , "Failed to sign in"))
            .map_err(|_| RetiscopeError::FailedToSignIn)?;

        // set the correct
        self.connection
            .use_ns("main")
            .use_db("main")
            .await
            .inspect_err(|e| error!(error = %e, "Failed to connect to database"))
            .map_err(|_| RetiscopeError::FailedToConnectToDB)?;

        {
            let _ = self.connection
            .query(
                r#"
                    -- 1. Unique Nodes
                    DEFINE TABLE node SCHEMAFULL;
                    DEFINE FIELD first_seen ON TABLE node TYPE datetime DEFAULT time::now();
                    DEFINE FIELD last_seen  ON TABLE node TYPE datetime DEFAULT time::now();
                    DEFINE INDEX node_addr  ON TABLE node COLUMNS id UNIQUE;
                    
                    -- 2. Announce Events (The Log)
                    DEFINE TABLE announce SCHEMAFULL;
                    DEFINE FIELD destination    ON TABLE announce TYPE record<node>;
                    DEFINE FIELD transport_node ON TABLE announce TYPE option<record<node>>;
                    DEFINE FIELD iface          ON TABLE announce TYPE string;
                    DEFINE FIELD hops           ON TABLE announce TYPE int;
                    DEFINE FIELD timestamp      ON TABLE announce TYPE datetime DEFAULT time::now();
                    
                    -- Index for fast "Give me history for Node X" queries
                    DEFINE INDEX announce_dest ON TABLE announce COLUMNS destination;

                    -- For the "last_seen" logic in the UPSERT
                    DEFINE INDEX idx_node_id ON TABLE node COLUMNS id UNIQUE;
                    
                    -- For the "history" and deduplication check
                    -- This makes 'WHERE destination = ... ORDER BY timestamp' instant
                    DEFINE INDEX idx_announce_lookup ON TABLE announce COLUMNS destination, timestamp;

                "#,
            )
            .await
            .inspect_err(|e| error!(error = %e, "Failed to send query"))
            .map_err(|_| RetiscopeError::FailedToSendQuery)?;
        }

        Ok(())
    }

    #[instrument(skip(self, data))]
    async fn save_announces(&self, data: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError> {
        if data.is_empty() {
            debug!("Announce Entries are empty. Nothing to give to the db");
            return Ok(());
        }
        // clears the data
        let count = data.len();
        let entries = std::mem::take(data);

        let query = r#"
            FOR $entry IN $data {
                -- Update 'last_seen' for the destination
                UPSERT type::record("node", $entry.destination) 
                SET last_seen = time::now();
    
                -- Update 'last_seen' for the transport_node
                IF $entry.transport_node != NONE {
                    UPSERT type::record("node", $entry.transport_node) 
                    SET last_seen = time::now();
                };
    
                -- Dump announce into db
                CREATE announce SET
                    destination = type::record("node", $entry.destination),
                    transport_node = IF $entry.transport_node != NONE { 
                        type::record("node", $entry.transport_node) 
                    } ELSE { 
                        NONE 
                    },
                    hops = $entry.hops,
                    iface = $entry.iface,
                    timestamp = time::now();
             };
        "#;

        // send query and log result
        let json_value: serde_json::Value =
            serde_json::to_value(&entries).map_err(|_| RetiscopeError::FailedQuery)?;
        self.connection
            .query(query)
            .bind(("data", json_value))
            .await
            // check if it failed to send
            .inspect_err(|e| error!(error = %e, "Failed to send batch query"))
            // check if the query executed properly
            .and_then(|response| {
                response
                    .check()
                    .inspect(|_| debug!(count = count, "Batch sync complete"))
                    .inspect_err(|e| error!(error = %e, "Batch query execution failed"))
            })
            .map_err(|_| RetiscopeError::FailedQuery)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn watch_announces(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<StoredAnnounce>, RetiscopeError> {
        // I am very unhappy with this LIVE SELECT query but
        // surrealdb KEEPS COMPLAINING whenever I try to use
        // the .live(), because of StoredAnnounce not
        // implementing SurrealValue. But I can't impl it for
        // AddressHash since it is a foreign type. I really don't
        // want to make even more wrappers.
        let mut stream = self
            .connection
            .query("LIVE SELECT *, type::string(id) AS id FROM announce")
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to establish live query");
                RetiscopeError::FailedQuery
            })?
            // surrealdb always returns a vec, even if it is only one element hence the '0'
            .stream::<surrealdb::Notification<surrealdb_types::Value>>(0)
            .map_err(|e| {
                error!(error = ?e, "Failed to convert live query into a stream");
                RetiscopeError::FailedQuery
            })?;

        // creates a channel and a background task to subscribe to updates
        let (tx, rx) = mpsc::unbounded::<StoredAnnounce>();
        tokio::spawn(async move {
            // errors and more erros
            while let Some(result) = stream.next().await {
                match result {
                    Ok(notification) => {
                        // convert the data
                        let norm = notification.data.into_json_value();
                        match serde_json::from_value::<StoredAnnounce>(norm) {
                            Ok(announce) => {
                                if let Err(_) = tx.unbounded_send(announce) {
                                    error!("Failed to send on channel");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to parse json into StoredAnnounce");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = ?e, "Stream error");
                    }
                }
            }
        });

        Ok(rx)
    }
    #[instrument(skip(self))]
    async fn watch_nodes(&self) -> Result<mpsc::UnboundedReceiver<StoredNode>, RetiscopeError> {
        // I am very unhappy with this LIVE SELECT query but
        // surrealdb KEEPS COMPLAINING whenever I try to use
        // the .live(), because of StoredAnnounce not
        // implementing SurrealValue. But I can't impl it for
        // AddressHash since it is a foreign type. I really don't
        // want to make even more wrappers.
        let mut stream = self
            .connection
            .query("LIVE SELECT *, type::string(id) AS id FROM node")
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to establish live query");
                RetiscopeError::FailedQuery
            })?
            // surrealdb always returns a vec, even if it is only one element hence the '0'
            .stream::<surrealdb::Notification<surrealdb_types::Value>>(0)
            .map_err(|e| {
                error!(error = ?e, "Failed to convert live query into a stream");
                RetiscopeError::FailedQuery
            })?;

        // creates a channel and a background task to subscribe to updates
        let (tx, rx) = mpsc::unbounded::<StoredNode>();
        tokio::spawn(async move {
            // errors and more erros
            while let Some(result) = stream.next().await {
                match result {
                    Ok(notification) => {
                        // convert the data
                        let norm = notification.data.into_json_value();
                        match serde_json::from_value::<StoredNode>(norm) {
                            Ok(announce) => {
                                if let Err(_) = tx.unbounded_send(announce) {
                                    error!("Failed to send on channel");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to parse json into StoredAnnounce");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = ?e, "Stream error");
                    }
                }
            }
        });

        Ok(rx)
    }
}

impl SurrealImpl {
    pub async fn new(
        address: &str,
        port: &u16,
        use_tls: bool,
        namespace: &str,
        database: &str,
    ) -> Result<Self, RetiscopeError> {
        let protocol = if use_tls { "wss" } else { "ws" };
        let endpoint = format!("{}://{}:{}", protocol, address, port);

        // the 'connect' function from the 'any' engine is magic
        let db = connect(endpoint)
            .await
            .map_err(|_| RetiscopeError::FailedToConnectToDB)?;

        db.use_ns(namespace)
            .use_db(database)
            .await
            .map_err(|_| RetiscopeError::FailedToConfigureDB)?;

        Ok(Self { connection: db })
    }
}
