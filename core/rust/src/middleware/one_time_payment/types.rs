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
    pub amount: U256,                // amount for the one-time payment
    pub period_ttl_sec: Option<u64>, // session validity period after payment
    pub rpc_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePayment {
    pub tx_hash: FixedBytes<32>,
    pub sender: Address,
    pub payment_timestamp: u64, // timestamp of the payment
    pub first_reedemed: u64,    // timestamp of first redemption
    pub redemptions: u32,       // number of times this payment has been redeemed
}

pub const ABS_WINDOW_SEC: u64 = 172800; // 2 days i.e. payment must be within this window
pub const SESSION_TTL_SEC: u64 = 3600; // 1 hour i.e. session is valid for this period once created
pub const MAX_REDEMPTIONS: u32 = 3; // max 3 redemption attempts allowed per payment
