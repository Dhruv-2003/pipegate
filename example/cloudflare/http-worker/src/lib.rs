use http::StatusCode;
use pipegate::channel::ChannelState;
use std::time::{SystemTime, UNIX_EPOCH};
use worker::*;

#[event(fetch)]
async fn fetch(_req: HttpRequest, _env: Env, _ctx: Context) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();

    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Body::empty())?)
}

// fn create_error_response(msg: &str, status: StatusCode) -> HttpResponse {
//     http::Response::builder()
//         .status(status)
//         .body(Body::new(msg))
//         .unwrap()
// }

// return a bool regarding the auth status or an error
async fn handle_request_auth(state: &ChannelState, request: HttpRequest) -> Result<bool> {
    // decode the request
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

    // prepare a signed request object

    // return the true or false
    Ok(true)
}
