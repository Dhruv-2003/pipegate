use alloy::{
    hex,
    primitives::{Bytes, PrimitiveSignature, U256},
};
use http::StatusCode;
use pipegate::{
    channel::ChannelState,
    types::{PaymentChannel, SignedRequest},
    verify::verify_and_update_channel,
};
use std::time::{SystemTime, UNIX_EPOCH};
use worker::*;

#[event(fetch)]
async fn fetch(_req: HttpRequest, _env: Env, _ctx: Context) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();

    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());
    let payment_amount = U256::from(1000);
    let request = _req;

    // Decode all the info from headers

    let timestamp = request
        .headers()
        .get("X-Timestamp")
        .and_then(|t| t.to_str().ok())
        .and_then(|t| t.parse::<u64>().ok())
        .unwrap();

    println!("Timestamp: {}", timestamp);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now - timestamp > 300 {
        return Ok(http::Response::builder()
            .status(http::StatusCode::REQUEST_TIMEOUT)
            .body(Body::empty())?);
    }

    let signature = request
        .headers()
        .get("X-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)
        .unwrap()
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .unwrap();

    let message = request
        .headers()
        .get("X-Message")
        .ok_or(StatusCode::UNAUTHORIZED)
        .unwrap()
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .unwrap();

    let payment_data = request
        .headers()
        .get("X-Payment")
        .ok_or(StatusCode::UNAUTHORIZED)
        .unwrap()
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)
        .unwrap();

    // parse the data

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
        })
        .unwrap();

    let message = hex::decode(message)
        .map_err(|_| {
            println!("Failed: Message decode");
            StatusCode::BAD_REQUEST
        })
        .unwrap();

    let payment_channel: PaymentChannel = serde_json::from_str(payment_data)
        .map_err(|e| {
            println!("Failed: Payment data decode - Error {}", e);
            StatusCode::BAD_REQUEST
        })
        .unwrap();

    // Convert body_bytes into a `Bytes` object or process as needed
    // TODO: Implement this
    let mut _body = request.into_body();
    let body_bytes = Bytes::from("0x");

    // prepare a signed request
    let signed_request = SignedRequest {
        message,
        signature,
        payment_channel,
        payment_amount,
        body_bytes: body_bytes.to_vec(),
    };

    // verify and update the channel
    let _updated_channel = verify_and_update_channel(&state, signed_request)
        .await
        .unwrap();

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Body::empty())?)
}
