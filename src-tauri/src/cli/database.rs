use tracing::{debug, error, info, instrument, trace, warn};

use reticulum::hash::AddressHash;

use serde::{Deserialize, Serialize};

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};
use surrealdb_types::SurrealValue;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnnounceData {
    pub hops: u8,
    pub transport_node: Option<AddressHash>,
    pub destination: AddressHash,
    pub iface: AddressHash,
}

#[derive(Serialize, SurrealValue)]
struct DbAnnounce {
    hops: u8,
    transport_node: Option<String>,
    destination: String,
    iface: String,
}

impl From<AnnounceData> for DbAnnounce {
    fn from(data: AnnounceData) -> Self {
        Self {
            hops: data.hops,
            transport_node: data.transport_node.map(|h| h.to_hex_string()),
            destination: data.destination.to_hex_string(),
            iface: data.iface.to_hex_string(),
        }
    }
}

pub async fn init() -> Surreal<Client> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .unwrap();
    // db.select("announce").live()
    db.use_ns("main").use_db("main").await.unwrap();
    {
        let _ = db
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
            .unwrap();
    }
    db
}

async fn flush_to_db(db: &Surreal<Client>, batch: &mut Vec<AnnounceData>) {
    if batch.is_empty() {
        debug!("Announce Entries are empty. Nothing to give to the db");
        return;
    }

    let original_data = std::mem::take(batch);

    // par_iter might be interesting here, not sure of the performance benefits tho
    let data_to_send: Vec<DbAnnounce> = original_data.into_iter().map(DbAnnounce::from).collect();

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

    match db.query(query).bind(("data", data_to_send)).await {
        Ok(response) => {
            // check for errors in the response
            if let Err(e) = response.check() {
                error!(error = %e, "Batch query execution failed");
            } else {
                trace!("Batch sync complete");
            }
        }
        Err(e) => error!(error = %e, "Failed to send batch query"),
    }
    batch.clear();
}
