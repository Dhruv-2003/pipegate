use alloy::primitives::U256;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    routing::get,
    Router,
};
use pipegate::{
    channel::ChannelState,
    error::AuthError,
    utils::{modify_headers_axum, parse_headers_axum},
    verify::verify_and_update_channel,
};
use tower_service::Service;
use worker::*;

fn router() -> Router {
    let state = ChannelState::new("https://base-sepolia-rpc.publicnode.com".parse().unwrap());
    let payment_amount = U256::from(1000);

    Router::new().route("/", get(root))
    // .layer(axum::middleware::from_fn(move |request, next| {
    //     let state = state.clone();
    //     auth_middleware(state, payment_amount, request, next)
    // }));
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router().call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}

pub async fn auth_middleware(
    state: ChannelState,
    payment_amount: U256, // defined by the developer creating the API, and should match with what user agreed with in the signed request
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let (signed_request, parts) = parse_headers_axum(request, payment_amount).await?;
    let body_bytes = signed_request.body_bytes.clone();

    let updated_channel = verify_and_update_channel(&state, signed_request).await?;

    let mut response = next.run(request).await;

    let response = modify_headers_axum(response, &updated_channel);

    Ok(response)
}
