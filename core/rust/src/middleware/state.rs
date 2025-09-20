use crate::middleware::{
    one_time_payment::state::OneTimePaymentState,
    payment_channel::channel::ChannelState,
    stream_payment::{
        state::StreamState,
        types::{StreamListenerConfig, StreamsConfig},
        StreamListner,
    },
    utils::{get_cfa_from_chain_id, get_chain_wss_url},
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MiddlewareState {
    pub stream_state: Arc<RwLock<Option<StreamState>>>,
    pub channel_state: Arc<RwLock<Option<ChannelState>>>,
    pub one_time_payment_state: Arc<RwLock<Option<OneTimePaymentState>>>,
}

impl MiddlewareState {
    pub fn new() -> Self {
        Self {
            stream_state: Arc::new(RwLock::new(None)),
            channel_state: Arc::new(RwLock::new(None)),
            one_time_payment_state: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn with_stream_state(self) -> Self {
        let mut stream_state = self.stream_state.write().await;
        *stream_state = Some(StreamState::new());
        drop(stream_state);
        self
    }

    pub async fn start_stream_listener(&self, chain_id: u64, stream_config: &StreamsConfig) {
        // get cfa address
        let cfa = match get_cfa_from_chain_id(&chain_id).await {
            Ok(addr) => addr,
            Err(e) => {
                println!("Error fetching CFA address: {:?}", e);
                return;
            }
        };

        // get wss url
        let wss_url = match get_chain_wss_url(&chain_id).await {
            Ok(url) => url,
            Err(e) => {
                println!("Error fetching WSS URL: {:?}", e);
                return;
            }
        };

        let stream_listener_config = StreamListenerConfig { wss_url, cfa };

        let stream_state_guard = self.stream_state.read().await;
        if let Some(stream_state) = stream_state_guard.as_ref() {
            let _listener = StreamListner::new(
                stream_state.clone(),
                stream_config.clone(),
                stream_listener_config,
            )
            .await;
        }
    }

    pub async fn with_channel_state(self) -> Self {
        let mut channel_state = self.channel_state.write().await;
        *channel_state = Some(ChannelState::new());
        drop(channel_state);
        self
    }

    pub async fn with_one_time_payment_state(self) -> Self {
        let mut one_time_payment_state = self.one_time_payment_state.write().await;
        *one_time_payment_state = Some(OneTimePaymentState::new());
        drop(one_time_payment_state);
        self
    }
}
