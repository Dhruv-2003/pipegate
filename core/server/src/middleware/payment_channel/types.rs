use alloy::primitives::{Address, PrimitiveSignature, U256};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentChannel {
    pub address: Address,
    pub sender: Address,
    pub recipient: Address,

    #[serde_as(as = "DisplayFromStr")]
    pub balance: U256,

    #[serde_as(as = "DisplayFromStr")]
    pub nonce: U256,

    #[serde_as(as = "DisplayFromStr")]
    pub expiration: U256,

    #[serde_as(as = "DisplayFromStr")]
    pub channel_id: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedRequest {
    pub message: Vec<u8>,
    pub signature: PrimitiveSignature,
    pub payment_channel: PaymentChannel,
    pub payment_amount: U256,
    pub body_bytes: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentChannelConfig {
    pub recipient: Address,
    pub token_address: Address,
    pub amount: U256, // amount for the one-time payment
    pub rpc_url: String,
}
