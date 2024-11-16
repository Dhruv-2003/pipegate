// Channel struct and implementation
// It's the local channel state for the middleware on the server side on how to store the info and just work with it

use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::providers::Provider;
use alloy::{
    primitives::{Address, U256},
    signers::Signature,
};
use tokio::sync::RwLock;

use crate::{error::AuthError, types::PaymentChannel};

#[derive(Clone)]
pub struct ChannelState {
    pub(crate) channels: Arc<RwLock<HashMap<U256, PaymentChannel>>>, // All the channels the current server has with other user
    rate_limiter: Arc<RwLock<HashMap<Address, (u64, SystemTime)>>>,  // Rate limiter for the user
                                                                     // provider: Arc<dyn Provider>, // Provider to interact with the blockchain
}

impl ChannelState {
    pub fn new(provider: impl Provider) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            // provider: Arc::new(provider),
        }
    }

    // verification method
    pub async fn verify_signature(
        &self,
        channel_id: U256,
        signature: &Signature,
        message: &[u8],
    ) -> Result<Address, AuthError> {
        // self.network.verify_signature(signature, message).await

        // Get the address for sender from the channel state using channel id
        let channel = self.channels.read().await;
        let payment_channel = channel.get(&channel_id).ok_or(AuthError::ChannelNotFound)?;

        // Network logic to verify the signature, could be a simple ECDSA verification
        let recovered = signature.recover_address_from_msg(message);

        // Match the recovered address with the one in the channel state
        match recovered {
            Ok(address) if address == payment_channel.sender => Ok(address),
            _ => Err(AuthError::InvalidSignature),
        }
    }

    // No need in theory for validating this at the start, can do but it will add latency for a contract call on each and every API, not good
    // Can do this the first time the channel is added to the local state
    pub async fn validate_channel(
        &self,
        channel_id: U256,
        payment_channel: &PaymentChannel,
    ) -> Result<(), AuthError> {
        // self.network.validate_channel(channel_id, balance).await

        // Fetch the balance for this payment channel from the contract implementation on the blockchain

        // If the balance is less than the balance in the local state, return an error
        // if payment_channel.balance < balance {
        //     return Err(AuthError::InsufficientFunds);
        // }

        // Fetch Expiration time for the channel from the contract

        // Verify other data from the contract as well for the payment channel

        Ok(())
    }

    // rate limiter method
    // ✅
    pub(crate) async fn check_rate_limit(&self, sender: Address) -> Result<(), AuthError> {
        const RATE_LIMIT: u64 = 100; // 100 requests
        const WINDOW: u64 = 60; // Every 60 seconds

        let mut rate_limits = self.rate_limiter.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (count, last_reset) = rate_limits.entry(sender).or_insert((0, SystemTime::now()));

        let last_reset_secs = last_reset.duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now - last_reset_secs >= WINDOW {
            *count = 1;
            *last_reset = SystemTime::now();
            Ok(())
        } else if *count >= RATE_LIMIT {
            Err(AuthError::RateLimitExceeded)
        } else {
            *count += 1;
            Ok(())
        }
    }
}

// update method - using the insert method on the HashMap directly
