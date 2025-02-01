use std::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::{
    dyn_abi::DynSolType,
    hex::{self},
    primitives::{Address, Signed, U256},
    providers::{Provider, ProviderBuilder},
    sol,
    transports::http::reqwest::Url,
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;
use tracing::info;

use crate::{
    benchmark::log_benchmark,
    channel::ChannelState,
    error::AuthError,
    types::{
        tx::{SignedStream, StreamsConfig},
        OneTimePaymentConfig, PaymentChannel, SignedPaymentTx, SignedRequest,
    },
    utils::{create_channel_message, create_tx_message, helpers::create_stream_message},
};

pub async fn verify_and_update_channel(
    state: &ChannelState,
    mut request: SignedRequest,
) -> Result<PaymentChannel, AuthError> {
    println!("\n=== verify_and_update_channel ===");
    println!("Payment amount: {}", request.payment_amount);
    println!(
        "Payment channel: {}",
        serde_json::to_string(&request.payment_channel).map_err(|_| AuthError::InternalError)?
    );
    println!("Channel balance: {}", request.payment_channel.balance);

    println!("Message length: {}", request.message.len());
    println!("Original message: 0x{}", hex::encode(&request.message));

    // Check timestamp first
    #[cfg(not(target_arch = "wasm32"))]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    #[cfg(target_arch = "wasm32")]
    let now = (Date::now() as u64) / 1000;

    if now - request.timestamp > 300 {
        return Err(AuthError::TimestampError);
    }

    let sig_start = Instant::now();
    // Verify that the message matches what we expect
    let reconstructed_message = create_channel_message(
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

    info!("Signature verification took {:?}", sig_start.elapsed());
    log_benchmark(
        "Signature verification",
        sig_start.elapsed().as_millis(),
        "channel",
    );

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

        let onchain_validation_start = Instant::now();
        state.validate_channel(&request.payment_channel).await?;
        info!(
            "On-chain validation took {:?}",
            onchain_validation_start.elapsed()
        );
        log_benchmark(
            "On-chain validation",
            onchain_validation_start.elapsed().as_millis(),
            "channel",
        );

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

// Verify the channel and return the updated channel
// No need of a state object here as it's just a verification
pub async fn verify_channel(
    rpc_url: Url,
    mut request: SignedRequest,
    current_channel: Option<PaymentChannel>,
) -> Result<PaymentChannel, AuthError> {
    println!("\n=== verify_channel ===");
    println!("Payment amount: {}", request.payment_amount);
    println!(
        "Payment channel: {}",
        serde_json::to_string(&request.payment_channel).map_err(|_| AuthError::InternalError)?
    );
    println!("Channel balance: {}", request.payment_channel.balance);

    println!("Message length: {}", request.message.len());
    println!("Original message: 0x{}", hex::encode(&request.message));

    // Verify that the message matches what we expect
    let reconstructed_message = create_channel_message(
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

    // Create a temporary state object
    let state = ChannelState::new(rpc_url);

    // If currentChannel is present add it to the state
    if let Some(channel) = current_channel {
        state
            .channels
            .write()
            .await
            .insert(channel.channel_id, channel);
    }

    // Verify signature using network-specific logic
    state
        .verify_signature(
            &request.payment_channel,
            &request.signature,
            &request.message,
        )
        .await?;

    let channels = state.channels.write().await;

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

    println!("API request authorized");
    Ok(request.payment_channel.clone())
}

// For one time payment verification
pub async fn verify_tx(
    signed_tx: SignedPaymentTx,
    config: OneTimePaymentConfig,
) -> Result<bool, AuthError> {
    let sig_start = Instant::now();
    // creating the message
    let reconstructed_message = create_tx_message(signed_tx.tx_hash);
    println!("Message: 0x{}", hex::encode(&reconstructed_message));

    let signature = signed_tx.signature;
    println!("Signature: 0x{}", hex::encode(&signature.as_bytes()));

    // recovering the address from the signature
    let recovered = match signature.recover_address_from_msg(reconstructed_message) {
        Ok(address) => address,
        Err(_) => return Err(AuthError::InvalidSignature),
    };
    println!("Recovered address: {}", recovered);
    info!("Signature verification took {:?}", sig_start.elapsed());
    log_benchmark(
        "Signature verification",
        sig_start.elapsed().as_millis(),
        "onetime",
    );

    let onchain_start = Instant::now();
    // Creating the provider
    let provider = ProviderBuilder::new().on_http(config.rpc_url.parse().unwrap());

    // Fetching the info for transaction
    let tx_receipt = match provider
        .get_transaction_receipt(signed_tx.tx_hash)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
    {
        Some(tx_receipt) => tx_receipt,
        None => {
            println!("Failed: Transaction not found");
            return Err(AuthError::TransactionNotFound);
        }
    };

    // Verifying recovered address against the sender for the transaction
    if recovered != tx_receipt.from {
        println!("Failed: Recovered address mismatch");
        return Err(AuthError::InvalidSignature);
    }

    // Match the contract interacted with to be the token contract
    match tx_receipt.to {
        Some(to) => {
            if to != config.token_address {
                return Err(AuthError::InvalidTransaction(
                    "Invalid token contract address".to_string(),
                ));
            }
        }
        None => {
            println!("Failed: To address not found");
            return Err(AuthError::InvalidTransaction(
                "To address not found".to_string(),
            ));
        }
    }

    let receipt = match tx_receipt.inner.as_receipt() {
        Some(receipt) => receipt,
        None => {
            return Err(AuthError::InvalidTransaction(
                "Receipt not found".to_string(),
            ))
        }
    };

    let transfer_log = match receipt.logs.first() {
        Some(log) => log,
        None => return Err(AuthError::InvalidTransaction("Log not found".to_string())),
    };

    // Check if the log is a transfer log & verify the topics and data
    if transfer_log.address() == config.token_address {
        match transfer_log.topics().get(2) {
            Some(t) => {
                let to = Address::from_word(t.clone());
                let data = &transfer_log.data().data;
                let data_type = DynSolType::Uint(256);
                let decoded = data_type
                    .abi_decode(&data)
                    .map_err(|e| AuthError::ContractError(e.to_string()))?;

                let (amount, _) = match decoded.as_uint() {
                    Some(amount) => amount,
                    None => {
                        return Err(AuthError::InvalidTransaction(
                            "Amount couldn't be parsed from event".to_string(),
                        ))
                    }
                };

                if to != config.recipient || amount != config.amount {
                    return Err(AuthError::InvalidTransaction(
                        "Invalid recipient or amount".to_string(),
                    ));
                }
            }
            None => return Err(AuthError::InvalidTransaction("Topic not found".to_string())),
        }
    }
    info!("On-chain call took {:?}", onchain_start.elapsed());
    log_benchmark(
        "On-chain call",
        onchain_start.elapsed().as_millis(),
        "onetime",
    );

    Ok(true)
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    CFAv1Forwarder,
    "src/abi/CFAv1Forwarder.json"
);

pub async fn verify_stream(stream: SignedStream, config: StreamsConfig) -> Result<bool, AuthError> {
    let sig_verify_start = Instant::now();
    // Creating the message
    let reconstructed_message = create_stream_message(stream.sender);
    println!("Message: 0x{}", hex::encode(&reconstructed_message));

    let signature = stream.signature;
    println!("Signature: 0x{}", hex::encode(&signature.as_bytes()));

    // Recovering the address from the signature
    let recovered = match signature.recover_address_from_msg(reconstructed_message) {
        Ok(address) => address,
        Err(_) => return Err(AuthError::InvalidSignature),
    };
    println!("Recovered address: {}", recovered);

    // Verify the recovered address against the sender for the stream
    if recovered != stream.sender {
        println!("Failed: Recovered address mismatch");
        return Err(AuthError::InvalidSignature);
    }
    info!(
        "Signature verification took {:?}",
        sig_verify_start.elapsed()
    );
    log_benchmark(
        "Signature verification",
        sig_verify_start.elapsed().as_millis(),
        "streams",
    );

    let onchain_start = Instant::now();
    let provider = ProviderBuilder::new().on_http(config.rpc_url.parse().unwrap());

    let cfav1_forwarder = CFAv1Forwarder::new(config.cfa_forwarder, provider);

    // Fetch the stream flow from sender to recipient, if it exists, using CFAv1Forwarder
    let flow_info = cfav1_forwarder
        .getFlowInfo(config.token_address, stream.sender, config.recipient)
        .call()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;
    info!("On-chain call took {:?}", onchain_start.elapsed());
    log_benchmark(
        "On-chain call",
        onchain_start.elapsed().as_millis(),
        "streams",
    );

    // Check if the flow exists
    if flow_info.flowrate == Signed::ZERO {
        println!("Failed: No stream flow found");
        return Err(AuthError::InvalidStream("No stream flow found".to_string()));
    } else {
        println!("Stream flow found");
        println!("Flow rate: {}", flow_info.flowrate);
        // check the flowRate matches with what recipient expects
        if flow_info.flowrate != config.amount {
            println!("Failed: Invalid stream flow rate");
            return Err(AuthError::InvalidStream(
                "Invalid stream flow rate".to_string(),
            ));
        }
    }

    Ok(true)
}
