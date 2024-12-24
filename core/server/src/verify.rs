#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::{hex, primitives::U256};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use crate::{
    channel::ChannelState,
    error::AuthError,
    types::{PaymentChannel, SignedRequest},
    utils::create_message,
};

pub async fn verify_and_update_channel(
    state: &ChannelState,
    mut request: SignedRequest,
) -> Result<PaymentChannel, AuthError> {
    println!("\n=== verify_and_update_channel ===");
    println!("Payment amount: {}", request.payment_amount);
    println!(
        "Payment channel: {}",
        serde_json::to_string(&request.payment_channel).unwrap()
    );
    println!("Channel balance: {}", request.payment_channel.balance);

    println!("Message length: {}", request.message.len());
    println!("Original message: 0x{}", hex::encode(&request.message));

    // Verify that the message matches what we expect
    let reconstructed_message = create_message(
        request.payment_channel.channel_id,
        request.payment_channel.balance,
        request.payment_channel.nonce,
        &request.body_bytes,
    );

    if request.message != reconstructed_message {
        println!("Failed: Message mismatch");
        return Err(AuthError::InvalidMessage);
    } else {
        println!("Message match");
    }

    // Verify signature using network-specific logic
    state
        .verify_signature(
            &request.payment_channel,
            &request.signature,
            &request.message,
        )
        .await?;

    let mut channels = state.channels.write().await;

    // Check if the channel is not expired with the current timestamp
    #[cfg(not(target_arch = "wasm32"))]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    #[cfg(target_arch = "wasm32")]
    let now = (Date::now() as u64) / 1000;

    if request.payment_channel.expiration < U256::from(now) {
        return Err(AuthError::Expired);
    }

    // Check if channel exists
    // NOTE: Nonce validation can be skipped as the balance will be acting as nonce here, the sender will always send the tx with the highest balance, we'll check for that here within our local record
    if let Some(existing_channel) = channels.get(&request.payment_channel.channel_id) {
        println!("Existing channel found");
        // Ensure new nonce is greater than existing nonce
        if request.payment_channel.nonce <= existing_channel.nonce {
            println!(
                "Failed: Invalid nonce - current: {}, received: {}",
                existing_channel.nonce, request.payment_channel.nonce
            );
            return Err(AuthError::InvalidNonce);
        } else {
            println!("Nonce match");
        }

        if request.payment_channel.balance != existing_channel.balance {
            println!(
                "Failed: Invalid balance - current: {}, received: {}",
                existing_channel.balance, request.payment_channel.balance
            );
            return Err(AuthError::InvalidChannel);
        } else {
            println!("Balance match");
        }
    } else {
        println!("New channel found");

        // Verify that the channel contract data is correct
        // 1. Verify the balance is available in the contract as the channel balance
        // 2. Verify the expiration is in the future
        // 3. Verify the channel ID is correct
        state.validate_channel(&request.payment_channel).await?;

        // Ensure the nonce is 0
        if request.payment_channel.nonce != U256::from(0) {
            return Err(AuthError::InvalidNonce);
        }
    }

    // NOTE: Update Balance for updating the local state, deducting the balance from the channel
    println!("Updating channel state");
    request.payment_channel.balance -= request.payment_amount;

    // Update or insert the channel
    channels.insert(
        request.payment_channel.channel_id,
        request.payment_channel.clone(),
    );

    println!("API request authorized");
    Ok(request.payment_channel.clone())
}
