use std::cell::RefCell;

use alloy::{
    primitives::{Address, U256},
    signers::Signature,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentChannel {
    pub channel_address: Address,
    pub sender: Address,
    pub recipient: Address,
    pub balance: U256,
    pub nonce: U256,
    pub expiration: U256,
    pub channel_id: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedRequest {
    pub message: Vec<u8>,
    pub signature: Signature,
    pub payment_channel: PaymentChannel,
    pub payment_amount: U256,
}
