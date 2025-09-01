use alloy::primitives::{aliases::I96, Address, PrimitiveSignature};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedStream {
    pub signature: PrimitiveSignature,
    pub sender: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamsConfig {
    pub rpc_url: String,
    pub cfa_forwarder: Address,
    pub token_address: Address,
    pub recipient: Address,
    pub amount: I96,     // flowRate for the stream, without decimals per second
    pub cache_time: u64, // in seconds
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stream {
    pub sender: Address,
    pub recipient: Address,
    pub token_address: Address,
    pub flow_rate: I96,
    pub last_verified: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamListenerConfig {
    pub wss_url: String,
    pub cfa: Address,
}
