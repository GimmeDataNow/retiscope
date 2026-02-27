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
might be the solution

ON DELETE CASCADE
