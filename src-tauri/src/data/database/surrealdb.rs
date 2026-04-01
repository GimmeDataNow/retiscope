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
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::data::{database::RetiscopeDB, AnnounceData, DBAnnounce};
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
    async fn set_up_db(&self) -> Result<(), RetiscopeError> {
        todo!()
    }
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
        let original_data = std::mem::take(data);

        let data_to_send: Vec<DBAnnounce> =
            original_data.into_iter().map(DBAnnounce::from).collect();

        // this was created in large part by gemini
        // seems alright tho
        let query = r#"
            FOR $entry IN $data {
                -- 1. Heartbeat for the Destination Node
                UPSERT type::record("node", $entry.destination) 
                SET last_seen = time::now();
    
                -- 2. Heartbeat for the Relay Node (if it exists)
                IF $entry.transport_node != NONE {
                    UPSERT type::record("node", $entry.transport_node) 
                    SET last_seen = time::now();
                };
    
                -- 3. Smart Announce Logic
                LET $dest_id = type::record("node", $entry.destination);
                LET $relay_id = IF $entry.transport_node != NONE { 
                    type::record("node", $entry.transport_node) 
                } ELSE { 
                    NONE 
                };
    
                LET $last = (
                    SELECT id, hops, transport_node, timestamp
                    FROM announce 
                    WHERE destination = $dest_id 
                    ORDER BY timestamp DESC 
                    LIMIT 1
                )[0];
    
                IF !$last OR $last.hops != $entry.hops OR $last.transport_node != $relay_id {
                    CREATE announce SET
                        destination = $dest_id,
                        transport_node = $relay_id,
                        hops = $entry.hops,
                        iface = $entry.iface,
                        timestamp = time::now();
                } ELSE {
                    UPDATE $last.id SET timestamp = time::now();
                };
            };
        "#;
        // send query and log result
        let count = data_to_send.len();
        self.connection
            .query(query)
            .bind(("data", data_to_send))
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

        // The 'connect' function from the 'any' engine is magic
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
