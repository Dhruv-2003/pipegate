use alloy::{
    hex::{self},
    primitives::Signed,
    providers::ProviderBuilder,
    sol,
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

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
