use crate::{
    error::AuthError,
    middleware::payment_channel::types::{PaymentChannel, SignedRequest},
};

use alloy::{
    dyn_abi::DynSolValue,
    hex::{self},
    primitives::{keccak256, PrimitiveSignature, U256},
};
use axum::{body::Body, http::Response};
use http::HeaderMap;

use std::time::{SystemTime, UNIX_EPOCH};

pub async fn parse_headers(
    headers: &HeaderMap,
    body_bytes: Vec<u8>,
    payment_amount: U256,
) -> Result<SignedRequest, AuthError> {
    // parse the request to retrieve the required headers
    let timestamp = headers
        .get("X-Timestamp")
        .and_then(|t| t.to_str().ok())
        .and_then(|t| t.parse::<u64>().ok())
        .ok_or(AuthError::MissingHeaders)?;

    println!("Timestamp: {}", timestamp);

    // Get and validate all required headers
    let signature = headers
        .get("X-Signature")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders)?;

    let message = headers
        .get("X-Message")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders)?;

    let payment_data = headers
        .get("X-Payment")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders)?;

    // Print all the headers
    println!("Signature: {}", signature);
    println!("Message: {}", message);
    println!("Payment Data: {}", payment_data);

    // Parse signature
    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| {
            println!("Failed: Signature decode");
            AuthError::InvalidSignature
        })
        .and_then(|bytes| {
            PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                println!("Failed: Signature conversion");
                AuthError::InvalidSignature
            })
        })?;

    // Parse message
    let message = hex::decode(message).map_err(|_| {
        println!("Failed: Message decode");
        AuthError::InvalidMessage
    })?;

    // Parse payment channel data
    let payment_channel: PaymentChannel = serde_json::from_str(payment_data).map_err(|e| {
        println!("Failed: Payment data decode - Error {}", e);
        AuthError::InvalidChannel
    })?;

    println!("Body: {}", String::from_utf8_lossy(&body_bytes));

    let signed_request = SignedRequest {
        message,
        signature,
        payment_channel,
        payment_amount,
        body_bytes: body_bytes,
        timestamp,
    };

    Ok(signed_request)
}

pub fn modify_headers_axum(
    mut response: Response<Body>,
    payment_channel: &PaymentChannel,
) -> Response<Body> {
    let headers_mut = response.headers_mut();

    // convert the payment channel json into string and then return that in the header
    headers_mut.insert(
        "X-Payment",
        serde_json::to_string(&payment_channel)
            .unwrap()
            .parse()
            .unwrap(),
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    headers_mut.insert("X-Timestamp", now.to_string().parse().unwrap());

    response
}

pub async fn modify_headers<B>(
    mut response: Response<B>,
    payment_channel: &PaymentChannel,
) -> http::Response<B> {
    let headers_mut = response.headers_mut();

    // convert the payment channel json into string and then return that in the header
    headers_mut.insert(
        "X-Payment",
        serde_json::to_string(&payment_channel)
            .unwrap()
            .parse()
            .unwrap(),
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    headers_mut.insert("X-Timestamp", now.to_string().parse().unwrap());

    response
}

pub fn create_channel_message(
    channel_id: U256,
    balance: U256,
    nonce: U256,
    body: &[u8],
) -> Vec<u8> {
    let message = DynSolValue::Tuple(vec![
        DynSolValue::Uint(channel_id, 256),
        DynSolValue::Uint(balance, 256),
        DynSolValue::Uint(nonce, 256),
        DynSolValue::Bytes(body.to_vec()),
    ]);

    let encoded_message = message.abi_encode_packed();

    let hashed_message = keccak256(&encoded_message);

    hashed_message.to_vec()
}
