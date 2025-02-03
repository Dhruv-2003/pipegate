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
    pub amount: I96, // flowRate for the stream, without decimals per second
}
