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

#[derive(Clone)]
pub struct MiddlewareState {
    pub stream_state: Option<StreamState>,
    pub channel_state: Option<ChannelState>,
    pub one_time_payment_state: Option<OneTimePaymentState>,
}

impl MiddlewareState {
    pub fn new() -> Self {
        Self {
            stream_state: None,
            channel_state: None,
            one_time_payment_state: None,
        }
    }

    pub fn with_stream_state(mut self) -> Self {
        self.stream_state = Some(StreamState::new());
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

        if let Some(stream_state) = &self.stream_state {
            let _listener = StreamListner::new(
                stream_state.clone(),
                stream_config.clone(),
                stream_listener_config,
            )
            .await;
        }
    }

    pub fn with_channel_state(mut self) -> Self {
        self.channel_state = Some(ChannelState::new());
        self
    }

    pub fn with_one_time_payment_state(mut self) -> Self {
        self.one_time_payment_state = Some(OneTimePaymentState::new());
        self
    }
}
