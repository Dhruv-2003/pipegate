use alloy::primitives::Address;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::middleware::stream_payment::Stream;

#[derive(Clone)]
pub struct StreamState {
    // Sender address to stream ( sender is the identifier )
    streams: Arc<RwLock<HashMap<Address, Stream>>>,
}

impl StreamState {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, stream_id: Address) -> Option<Stream> {
        let streams = self.streams.read().await;
        streams.get(&stream_id).cloned()
    }

    pub async fn set(&self, stream_id: Address, stream: Stream) {
        let mut streams = self.streams.write().await;
        streams.insert(stream_id, stream);
    }

    pub async fn invalidate(&self, stream_id: Address) {
        let mut streams = self.streams.write().await;
        streams.remove(&stream_id);
    }
}
