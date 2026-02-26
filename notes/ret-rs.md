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
