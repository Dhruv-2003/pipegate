// Channel struct and implementation
// It's the local channel state for the middleware on the server side on how to store the info and just work with it

use std::{collections::HashMap, sync::Arc};

use alloy::{
    contract::Error,
    network::EthereumWallet,
    primitives::{FixedBytes, PrimitiveSignature, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use alloy::{primitives::Bytes, transports::http::reqwest::Url};
use tokio::sync::RwLock;

use crate::{
    error::AuthError,
    middleware::payment_channel::types::{
        PaymentChannel as PaymentChannelType, PaymentChannelConfig,
    },
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    PaymentChannelABI,
    "src/abi/PaymentChannel.json"
);

#[derive(Clone)]
pub struct ChannelState {
    pub(crate) channels: Arc<RwLock<HashMap<U256, PaymentChannelType>>>, // All the channels the current server has with other user
    pub(crate) latest_signatures: Arc<RwLock<HashMap<U256, PrimitiveSignature>>>, // Latest signatures for each channel
}

impl ChannelState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            latest_signatures: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_channel(&self, channel_id: U256) -> Option<PaymentChannelType> {
        let channels = self.channels.read().await;
        channels.get(&channel_id).cloned()
    }

    /// Get the latest signature for a channel
    pub async fn get_latest_signature(&self, channel_id: U256) -> Option<PrimitiveSignature> {
        let signatures = self.latest_signatures.read().await;
        signatures.get(&channel_id).cloned()
    }

    /// Update the latest signature for a channel
    pub async fn update_latest_signature(&self, channel_id: U256, signature: PrimitiveSignature) {
        let mut signatures = self.latest_signatures.write().await;
        signatures.insert(channel_id, signature);
    }

    // verification method
    pub async fn verify_signature(
        &self,
        payment_channel: &PaymentChannelType,
        signature: &PrimitiveSignature,
        message: &[u8],
    ) -> Result<(), AuthError> {
        // self.network.verify_signature(signature, message).await

        // Network logic to verify the signature, could be a simple ECDSA verification
        let recovered = signature.recover_address_from_msg(message);
        println!("Recovered address: {:?}", recovered);

        // Match the recovered address with the one in the channel state
        match recovered {
            Ok(address) if address == payment_channel.sender => Ok(()),
            _ => {
                Err(AuthError::InvalidSignature)
                // NOTE : Ok(Address::default())
            }
        }
    }

    // Validating all the information of the channel from the onchain contract for the first time, before the channel is used
    pub async fn validate_channel(
        &self,
        config: &PaymentChannelConfig,
        payment_channel: &PaymentChannelType,
    ) -> Result<(), AuthError> {
        // self.network.validate_channel(channel_id, balance).await
        let provider = ProviderBuilder::new().on_http(config.rpc_url.parse().unwrap());

        let payment_channel_contract = PaymentChannelABI::new(payment_channel.address, provider);

        let channel_info = payment_channel_contract
            .getChannelInfo()
            .call()
            .await
            .map_err(|e| {
                AuthError::ContractError(format!("Failed to fetch channel info: {}", e.to_string()))
            })?;

        let balance = channel_info.balance;
        println!("Balance: {}", balance);

        // If the balance is less than the balance in the local state, return an error
        if payment_channel.balance < balance {
            return Err(AuthError::InsufficientBalance);
        }

        let expiration = channel_info.exp;
        println!("Expiration: {}", expiration);

        if payment_channel.expiration != expiration {
            return Err(AuthError::Expired);
        }

        // Verify the channelID from the contract
        let channel_id = channel_info.id;
        println!("Channel ID: {}", channel_id);

        if payment_channel.channel_id != channel_id {
            return Err(AuthError::InvalidChannel(format!(
                "Channel ID mismatch - expected: {}, received: {}",
                channel_id, payment_channel.channel_id
            )));
        }

        // Verify sender and recipient from the contract
        let sender_value = channel_info.senderAddr;

        if payment_channel.sender != sender_value {
            return Err(AuthError::InvalidChannel(format!(
                "Sender mismatch - expected: {}, received: {}",
                sender_value, payment_channel.sender
            )));
        }

        let recipient_value = channel_info.recipientAddr;

        if payment_channel.recipient != recipient_value
            || payment_channel.recipient != config.recipient
        {
            return Err(AuthError::InvalidChannel(format!(
                "Recipient mismatch - channel recipient: {}, contract recipient: {}, config recipient: {}",
                payment_channel.recipient, recipient_value, config.recipient
            )));
        }

        let price_value = channel_info.pricePerRequest;

        if price_value != config.amount {
            return Err(AuthError::InvalidChannel(format!(
                "Price mismatch - expected: {}, contract price: {}",
                config.amount, price_value
            )));
        }

        let token_value = payment_channel_contract
            .token()
            .call()
            .await
            .map_err(|e| AuthError::ContractError(e.to_string()))?
            ._0;

        if token_value != config.token_address {
            return Err(AuthError::InvalidChannel(format!(
                "Token address mismatch - expected: {}, contract token: {}",
                config.token_address, token_value
            )));
        }

        Ok(())
    }
}

// Close the channel using the latest signature stored in state
pub async fn close_channel_from_state(
    state: &ChannelState,
    rpc_url: Url,
    private_key: &str,
    channel_id: U256,
    raw_body: Bytes,
) -> Result<FixedBytes<32>, AuthError> {
    // Get the channel and latest signature from state
    let payment_channel = state
        .get_channel(channel_id)
        .await
        .ok_or_else(|| AuthError::ChannelNotFound)?;

    let signature = state
        .get_latest_signature(channel_id)
        .await
        .ok_or_else(|| AuthError::InvalidChannel("No signature found for channel".to_string()))?;

    // Close the channel
    close_channel(rpc_url, private_key, &payment_channel, &signature, raw_body)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))
}

// Close the channel to withdraw the funds
pub async fn close_channel(
    rpc_url: Url,
    private_key: &str,
    payment_channel: &PaymentChannelType,
    signature: &PrimitiveSignature,
    raw_body: Bytes,
) -> Result<FixedBytes<32>, Error> {
    let signer: PrivateKeySigner = private_key.parse().expect("Invalid private key");
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(rpc_url.clone());

    let payment_channel_contract = PaymentChannelABI::new(payment_channel.address, provider);

    let tx_hash = payment_channel_contract
        .close(
            payment_channel.balance,
            payment_channel.nonce,
            raw_body,
            Bytes::from(signature.as_bytes()),
        )
        .send()
        .await?
        .watch()
        .await?;

    Ok(tx_hash)
}
