use alloy::{
    hex,
    primitives::{FixedBytes, PrimitiveSignature, U256},
};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    http::{request::Parts, Request, Response, StatusCode},
};

use crate::types::{PaymentChannel, SignedPaymentTx, SignedRequest};

pub async fn parse_headers_axum(
    request: Request<axum::body::Body>,
    payment_amount: U256,
) -> Result<(SignedRequest, Parts), StatusCode> {
    // parse the request to retrieve the required headers
    // Check timestamp first
    let timestamp = request
        .headers()
        .get("X-Timestamp")
        .and_then(|t| t.to_str().ok())
        .and_then(|t| t.parse::<u64>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    println!("Timestamp: {}", timestamp);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now - timestamp > 300 {
        return Err(StatusCode::REQUEST_TIMEOUT);
    }

    // Get and validate all required headers
    let signature = request
        .headers()
        .get("X-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let message = request
        .headers()
        .get("X-Message")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let payment_data = request
        .headers()
        .get("X-Payment")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Print all the headers
    println!("Signature: {}", signature);
    println!("Message: {}", message);
    println!("Payment Data: {}", payment_data);

    // Parse signature
    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| {
            println!("Failed: Signature decode");
            StatusCode::BAD_REQUEST
        })
        .and_then(|bytes| {
            PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                println!("Failed: Signature conversion");
                StatusCode::BAD_REQUEST
            })
        })?;

    // Parse message
    let message = hex::decode(message).map_err(|_| {
        println!("Failed: Message decode");
        StatusCode::BAD_REQUEST
    })?;

    // Parse payment channel data
    let payment_channel: PaymentChannel = serde_json::from_str(payment_data).map_err(|e| {
        println!("Failed: Payment data decode - Error {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Get request body
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed: Body decode");
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    println!("Body: {}", String::from_utf8_lossy(&body_bytes));

    let signed_request = SignedRequest {
        message,
        signature,
        payment_channel,
        payment_amount,
        body_bytes: body_bytes.to_vec(),
    };

    Ok((signed_request, parts))
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

pub async fn parse_headers<B>(
    request: http::Request<B>,
    body_bytes: Vec<u8>,
    payment_amount: U256,
) -> Result<(SignedRequest, http::request::Parts), StatusCode>
where
    B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
{
    let timestamp = request
        .headers()
        .get("X-Timestamp")
        .and_then(|t| t.to_str().ok())
        .and_then(|t| t.parse::<u64>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    println!("Timestamp: {}", timestamp);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now - timestamp > 300 {
        return Err(StatusCode::REQUEST_TIMEOUT);
    }

    // Get and validate all required headers
    let signature = request
        .headers()
        .get("X-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let message = request
        .headers()
        .get("X-Message")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let payment_data = request
        .headers()
        .get("X-Payment")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Print all the headers
    println!("Signature: {}", signature);
    println!("Message: {}", message);
    println!("Payment Data: {}", payment_data);

    // Parse signature
    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| {
            println!("Failed: Signature decode");
            StatusCode::BAD_REQUEST
        })
        .and_then(|bytes| {
            PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                println!("Failed: Signature conversion");
                StatusCode::BAD_REQUEST
            })
        })?;

    // Parse message
    let message = hex::decode(message).map_err(|_| {
        println!("Failed: Message decode");
        StatusCode::BAD_REQUEST
    })?;

    // Parse payment channel data
    let payment_channel: PaymentChannel = serde_json::from_str(payment_data).map_err(|e| {
        println!("Failed: Payment data decode - Error {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Get request body
    let (parts, _body) = request.into_parts();

    // let body_bytes = match body::to_bytes(body, usize::MAX).await {
    //     Ok(bytes) => bytes,
    //     Err(_) => {
    //         println!("Failed: Body decode");
    //         return Err(StatusCode::BAD_REQUEST);
    //     }
    // };
    println!("Body: {}", String::from_utf8_lossy(&body_bytes));

    let signed_request = SignedRequest {
        message,
        signature,
        payment_channel,
        payment_amount,
        body_bytes: body_bytes.to_vec(),
    };

    Ok((signed_request, parts))
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

pub async fn parse_tx_headers_axum(
    request: &Request<axum::body::Body>,
) -> Result<SignedPaymentTx, StatusCode> {
    let signature = request
        .headers()
        .get("X-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| {
            println!("Failed: Signature decode");
            StatusCode::BAD_REQUEST
        })
        .and_then(|bytes| {
            PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                println!("Failed: Signature conversion");
                StatusCode::BAD_REQUEST
            })
        })?;

    let tx_hash = request
        .headers()
        .get("X-Transaction")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let tx_hash = hex::decode(tx_hash).map_err(|_| {
        println!("Failed: Message decode");
        StatusCode::BAD_REQUEST
    })?;

    let signed_tx = SignedPaymentTx {
        signature,
        tx_hash: FixedBytes::<32>::from_slice(&tx_hash),
    };

    Ok(signed_tx)
}
