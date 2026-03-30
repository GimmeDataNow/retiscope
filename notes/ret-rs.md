This is a function of the Transport
pub async fn request_path(
        &self,
        destination: &AddressHash,
        on_iface: Option<AddressHash>,
        tag: Option<TagBytes>,
    ) {
        self.handler.lock().await.request_path(destination, on_iface, tag).await
    }

the self.handler.lock().await returns the handler which contains the path table, announce table, the link table, the announce cache

-> copy the path table, announce table, announce cache, link table

send this to a database

This might need to support configuration for remote databases.
This might require to 'diff' the state. It would be best to do that with rayon/par_iter

500 elements might equate to ~55kb of data (sync every X seconds)
iter over the data. batch commit
#[serde(with = "serde_bytes")] might be need for binary data.
if the data is in binary format then manual search might be possible


pub struct PathTable {
    map: HashMap<AddressHash, PathEntry>,
}

pub struct PathEntry {
    pub timestamp: Instant,
    pub received_from: AddressHash,
    pub hops: u8,
    pub iface: AddressHash,
    pub packet_hash: Hash,
}

pub struct LinkTable(HashMap<LinkId, LinkEntry>);

pub struct LinkEntry {
    pub timestamp: Instant,
    pub proof_timeout: Instant,
    pub next_hop: AddressHash,
    pub next_hop_iface: AddressHash,
    pub received_from: AddressHash,
    pub original_destination: AddressHash,
    pub taken_hops: u8,
    pub remaining_hops: u8,
    pub validated: bool,
}

pub struct AnnounceTable {
    map: BTreeMap<AddressHash, AnnounceEntry>,
    responses: BTreeMap<AddressHash, AnnounceEntry>,
    cache: AnnounceCache,
}

pub struct AnnounceEntry {
    pub packet: Packet,
    pub timestamp: Instant,
    pub timeout: Instant,
    pub received_from: AddressHash,
    pub retries: u8,
    pub hops: u8,
    pub response_to_iface: Option<AddressHash>,
}

pub struct Packet {
    pub header: Header,
    pub ifac: Option<PacketIfac>,
    pub destination: AddressHash,
    pub transport: Option<AddressHash>,
    pub context: PacketContext,
    pub data: PacketDataBuffer,
}

pub struct PacketIfac {
    pub access_code: [u8; PACKET_IFAC_MAX_LENGTH],
    pub length: usize,
}

PacketContext is an enum

pub type PacketDataBuffer = StaticBuffer<PACKET_MDU>;

pub const PACKET_MDU: usize = 2048usize;

struct AnnounceCache {
    newer: Option<BTreeMap<AddressHash, AnnounceEntry>>,
    older: Option<BTreeMap<AddressHash, AnnounceEntry>>,
    capacity: usize
}





surreal start --allow-experimental record_references
surreal start --user a --pass a --bind 0.0.0.0:8000 rocksdb:$HOME/.local/share/retiscope/surreal/
might be the solution

ON DELETE CASCADE

pub struct AnnounceEvent {
    pub destination: Arc<Mutex<SingleOutputDestination>>,
    pub app_data: PacketDataBuffer,
}

pub type SingleOutputDestination = Destination<Identity, Output, Single>;

pub struct Destination<I: HashIdentity, D: Direction, T: Type> {
    pub direction: PhantomData<D>,
    pub r#type: PhantomData<T>,
    pub identity: I,
    pub desc: DestinationDesc,
}

pub struct DestinationDesc {
    pub identity: Identity,
    pub address_hash: AddressHash,
    pub name: DestinationName,
}

AnnouceEvent->Destination->DestinationDesc->(identity, address_hash, name)
identity->(public_key, verifying_key, address_hash)



The transport.recv_announces() function call is already cryptographically validated.
The handler.announce_tx.send() call is called from the handle_announce() function call.
It triggers at the end of the function but only if DestinationAnnounce::validate(packet)
has passed.

it may be neccesary to modify the handle_announce() function to allow reading of the hop count.
It is not provided natively. Modification to the AnnounceEvent will then also be needed.
It seems that AnnounceEvent is rarely used, thus making it a simple modification.


node->relation->node

relation:
    hops, time, 



use tauri state to pass the connection from the backend to the frontend
would look something like this:
```rust
#[tokio::main]
async fn main() {
    // 1. Init DB
    let db = cli::db_init().await; 
    
    // 2. Start your background router (Pass a clone of DB to it)
    let db_clone = db.clone();
    tokio::spawn(async move {
        cli::router_init(db_clone).await;
    });

    tauri::Builder::default()
        // 3. Manage the DB handle so Commands can see it
        .manage(db) 
        .invoke_handler(tauri::generate_handler![get_announces])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
```rust
use tauri::State;
use surrealdb::{Surreal, engine::remote::ws::Client};

#[tauri::command]
pub async fn get_announces(
    db: State<'_, Surreal<Client>>
) -> Result<Vec<AnnounceData>, String> {
    // Query the DB for the last 50 announces for the graph
    let mut response = db
        .query("SELECT * FROM path_table ORDER BY last_seen DESC LIMIT 50")
        .await
        .map_err(|e| e.to_string())?;

    let announces: Vec<AnnounceData> = response.take(0).map_err(|e| e.to_string())?;
    Ok(announces)
}
```

hops = 0 means it is on the same local hub (hearing your own announce)
hubs do not count as hops themselves if there a local node is trying to connect to somewhere else.
