use log::{self, info};

use rand_core::OsRng;
use reticulum::destination::{DestinationName, SingleInputDestination};
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use surrealdb::engine::remote::ws::{Ws, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};

pub async fn db_init() {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .unwrap();
    db.use_ns("main").use_db("main").await.unwrap();
    {
        // db.query("DEFINE FIELD timestamp ON TABLE path_table;")
        //     .await
        //     .unwrap();
        let _ = db
            .query(
                r#"
                    DEFINE TABLE path_table SCHEMAFULL;

                    DEFINE FIELD timestamp     ON TABLE path_table TYPE datetime;
                    DEFINE FIELD received_from ON TABLE path_table TYPE string;
                    DEFINE FIELD hops          ON TABLE path_table TYPE int;
                    DEFINE FIELD iface         ON TABLE path_table TYPE string;
                    DEFINE FIELD packet_hash   ON TABLE path_table TYPE string;
                "#,
            )
            .await
            .unwrap();
        let _ = db
            .query(
                r#"
                    DEFINE TABLE link_table SCHEMAFULL;

                    DEFINE FIELD timestamp            ON TABLE link_table TYPE datetime;
                    DEFINE FIELD proof_timeout        ON TABLE link_table TYPE datetime;
                    DEFINE FIELD next_hop             ON TABLE link_table TYPE string;
                    DEFINE FIELD next_hop_iface       ON TABLE link_table TYPE string;
                    DEFINE FIELD received_from        ON TABLE link_table TYPE string;
                    DEFINE FIELD original_destination ON TABLE link_table TYPE string;
                    DEFINE FIELD taken_hops           ON TABLE link_table TYPE int;
                    DEFINE FIELD remaining_hops       ON TABLE link_table TYPE int;
                    DEFINE FIELD validated            ON TABLE link_table TYPE bool DEFAULT false;
                "#,
            )
            .await
            .unwrap();
        let _ = db
            .query(
                r#"
                    DEFINE TABLE announce_map SCHEMAFULL;
                    DEFINE TABLE announce_responses SCHEMAFULL;
                    
                    DEFINE FIELD timestamp          ON TABLE announce_map, announce_responses TYPE datetime;
                    DEFINE FIELD timeout            ON TABLE announce_map, announce_responses TYPE datetime;
                    DEFINE FIELD received_from      ON TABLE announce_map, announce_responses TYPE string;
                    DEFINE FIELD retries            ON TABLE announce_map, announce_responses TYPE int;
                    DEFINE FIELD hops               ON TABLE announce_map, announce_responses TYPE int;
                    DEFINE FIELD response_to_iface  ON TABLE announce_map, announce_responses TYPE option<string>;
                    
                    DEFINE FIELD packet             ON TABLE announce_map, announce_responses TYPE object;
                    DEFINE FIELD packet.header      ON TABLE announce_map, announce_responses TYPE object;
                    DEFINE FIELD packet.destination ON TABLE announce_map, announce_responses TYPE string;
                    DEFINE FIELD packet.transport   ON TABLE announce_map, announce_responses TYPE option<string>;
                    DEFINE FIELD packet.context     ON TABLE announce_map, announce_responses TYPE object; -- Serialized Enum
                    DEFINE FIELD packet.data        ON TABLE announce_map, announce_responses TYPE string; -- Hex/Base64 string
                    DEFINE FIELD packet.ifac        ON TABLE announce_map, announce_responses TYPE option<object>;
                    
                    DEFINE TABLE announce_cache SCHEMAFULL;
                    
                    DEFINE FIELD age_group          ON TABLE announce_cache TYPE string 
                        ASSERT $value INSIDE ['newer', 'older'];
                    
                    DEFINE FIELD packet             ON TABLE announce_cache TYPE object;
                    DEFINE FIELD timestamp          ON TABLE announce_cache TYPE datetime;
                "#,
            )
            .await
            .unwrap();
    }
}

pub fn db_serve() {
    // let db = Surreal::new(address)
    // let db = Surreal::new::<SurrealKv>(".cache/retiscope/")
    // .versioned()
    // .await?;
}
