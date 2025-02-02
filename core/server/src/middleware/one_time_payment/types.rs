use alloy::primitives::{Address, FixedBytes, PrimitiveSignature, U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPaymentTx {
    pub signature: PrimitiveSignature,
    pub tx_hash: FixedBytes<32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePaymentConfig {
    pub recipient: Address,
    pub token_address: Address,
    pub amount: U256, // amount for the one-time payment
    pub period: U256,
    pub rpc_url: String,
}
