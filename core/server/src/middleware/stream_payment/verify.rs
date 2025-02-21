use alloy::{
    hex::{self},
    primitives::Signed,
    providers::ProviderBuilder,
    sol,
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;
use reqwest::Client;
use serde_json::json;

use crate::{
    error::AuthError,
    middleware::stream_payment::{
        types::{SignedStream, StreamsConfig},
        utils::create_stream_message,
    },
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    CFAv1Forwarder,
    "src/abi/CFAv1Forwarder.json"
);

#[allow(dead_code)]
pub(crate) async fn verify_stream_via_indexer(
    stream: SignedStream,
    config: StreamsConfig,
) -> Result<bool, AuthError> {
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

    let client = Client::new();

    let url = "https://subgraph-endpoints.superfluid.dev/base-sepolia/protocol-v1";

    let query = json!({
        "query": "query ($recipient: String!, $sender: String!, $amount: String!, $token: String!) {
            account(id: $recipient) {
                inflows(where: { sender: $sender, currentFlowRate: $amount, token: $token }) {
                    currentFlowRate
                    createdAtTimestamp
                    id
                }
            }
        }",
        "variables": {
            "recipient": config.recipient.to_string().to_lowercase(),
            "sender": stream.sender.to_string().to_lowercase(),
            "amount": config.amount.to_string(),
            "token": config.token_address.to_string().to_lowercase(),
        }
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await
        .map_err(|e| {
            println!("Failed: Network error: {}", e);
            AuthError::NetworkError("Failed to fetch stream data from indexer".to_string())
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            println!("Failed: JSON parse error: {}", e);
            AuthError::NetworkError("Failed to parse stream data from indexer".to_string())
        })?;

    if let Some(inflows) = response["data"]["account"]["inflows"].as_array() {
        if !inflows.is_empty() {
            println!("✅ Stream is active! Inflow record found.");
        } else {
            println!("❌ No active inflow detected.");
            return Err(AuthError::InvalidStream(
                "No active inflow detected".to_string(),
            ));
        }
    } else {
        println!("❌ No inflow data found.");
        return Err(AuthError::InvalidStream("No inflow data found".to_string()));
    }

    Ok(true)
}

pub async fn verify_stream(stream: SignedStream, config: StreamsConfig) -> Result<bool, AuthError> {
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

    let provider = ProviderBuilder::new().on_http(config.rpc_url.parse().unwrap());

    let cfav1_forwarder = CFAv1Forwarder::new(config.cfa_forwarder, provider);

    // Fetch the stream flow from sender to recipient, if it exists, using CFAv1Forwarder
    let flow_info = cfav1_forwarder
        .getFlowInfo(config.token_address, stream.sender, config.recipient)
        .call()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

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
