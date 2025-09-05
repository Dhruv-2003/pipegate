use crate::middleware::{
    one_time_payment::state::OneTimePaymentState, payment_channel::channel::ChannelState,
    stream_payment::state::StreamState,
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

    pub fn with_channel_state(mut self) -> Self {
        self.channel_state = Some(ChannelState::new());
        self
    }

    pub fn with_one_time_payment_state(mut self) -> Self {
        self.one_time_payment_state = Some(OneTimePaymentState::new());
        self
    }
}
