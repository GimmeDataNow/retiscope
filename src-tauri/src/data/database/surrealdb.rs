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
use futures::channel::mpsc;
use futures::{StreamExt, TryStreamExt};
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::data::{database::RetiscopeDB, AnnounceData, StoredAnnounce};
use crate::errors::RetiscopeError;

use async_trait::async_trait;

use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;

use serde_json::from_value;
use serde_json::Value;

#[derive(Debug)]
#[cfg(feature = "surrealdb")]
pub struct SurrealImpl {
    pub connection: surrealdb::Surreal<Any>,
}

#[derive(Debug, serde::Deserialize)]
struct AnnounceRow {
    id: String,
    // mirror all other fields from StoredAnnounce
    #[serde(flatten)]
    inner: serde_json::Value,
}

fn surreal_value_to_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            // Tagged surrealdb::Value variants
            if let Some(inner) = map.get("String") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Int") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Float") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Bool") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Datetime") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Number") {
                return surreal_value_to_json(inner);
            }
            if let Some(inner) = map.get("Null") {
                return serde_json::Value::Null;
            }
            // RecordId -> flatten to "table:key" string
            // if let Some(record) = map.get("RecordId") {
            //     if let serde_json::Value::Object(r) = record {
            //         let table = r.get("table").and_then(|t| t.as_str()).unwrap_or("");
            //         let key = r.get("key").map(surreal_value_to_json);
            //         let key_str = key.as_ref().and_then(|k| k.as_str()).unwrap_or("");
            //         return serde_json::Value::String(format!("{}:{}", table, key_str));
            //     }
            // }
            if let Some(record) = map.get("RecordId") {
                if let serde_json::Value::Object(r) = record {
                    let key = r.get("key").map(surreal_value_to_json);
                    // Just return the key, not "table:key"
                    return key.unwrap_or(serde_json::Value::Null);
                }
            }
            // Unwrap the "Object" wrapper
            if let Some(inner_obj) = map.get("Object") {
                return surreal_value_to_json(inner_obj);
            }
            // Plain object — recurse into values
            serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), surreal_value_to_json(v)))
                    .collect(),
            )
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(surreal_value_to_json).collect())
        }
        other => other.clone(),
    }
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
        let count = data.len();
        let entries = std::mem::take(data);

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
                error!(error = ?e, "watch announces failed");
                RetiscopeError::FailedQuery
            })?
            // surrealdb always returns a vec, even if it is only one element
            .stream::<surrealdb::Notification<surrealdb_types::Value>>(0)
            .map_err(|e| {
                error!(error = ?e, "watch announces failed");
                RetiscopeError::FailedQuery
            })?;

        // creates a channel and a background task to subscribe to updates
        let (tx, rx) = mpsc::unbounded::<StoredAnnounce>();
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(notification) => {
                        let mut norm = notification.data.into_json_value();
                        info!(?norm);
                        if let Some(obj) = norm.as_object_mut() {
                            for key in &["destination", "transport_node"] {
                                if let Some(serde_json::Value::String(s)) = obj.get(*key) {
                                    let stripped = s.splitn(2, ':').nth(1).unwrap_or(s).to_string();
                                    obj.insert(
                                        key.to_string(),
                                        serde_json::Value::String(stripped),
                                    );
                                }
                            }
                        }

                        match serde_json::from_value::<StoredAnnounce>(norm) {
                            Ok(announce) => info!("all good"),
                            Err(e) => {
                                error!(error = ?e, "no good");
                                break;
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
    async fn node_announces(&self) -> Result<(), RetiscopeError> {
        todo!()
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
