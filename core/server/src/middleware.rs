use std::{future::Future, pin::Pin};

use alloy::primitives::U256;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use serde_json::json;
use tower::{Layer, Service};

use crate::{
    channel::ChannelState,
    types::{OneTimePaymentConfig, PaymentChannel},
    utils::{headers::parse_tx_headers_axum, modify_headers_axum, parse_headers},
    verify::{verify_and_update_channel, verify_tx},
};

//* PAYMENT CHANNEL MIDDLEWARE LOGIC */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PipegateMiddlewareLayer {
    state: ChannelState,
    payment_amount: U256,
}

#[cfg(not(target_arch = "wasm32"))]
impl PipegateMiddlewareLayer {
    pub fn new(state: ChannelState, payment_amount: U256) -> Self {
        Self {
            state,
            payment_amount,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for PipegateMiddlewareLayer {
    type Service = PipegateMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        PipegateMiddleware {
            state: self.state.clone(),
            payment_amount: self.payment_amount,
            inner: service,
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PipegateMiddleware<S> {
    inner: S,
    state: ChannelState,
    payment_amount: U256,
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Service<Request<Body>> for PipegateMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    // #[cfg(target_arch = "wasm32")]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    // #[cfg(not(target_arch = "wasm32"))]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let payment_amount = self.payment_amount;
        let mut inner = self.inner.clone();

        // #[cfg(not(target_arch = "wasm32"))]
        Box::pin(async move {
            match check_payment_channel_middleware(state, payment_amount, request).await {
                Ok((request, payment_channel, state)) => {
                    state
                        .channels
                        .write()
                        .await
                        .insert(payment_channel.channel_id, payment_channel.clone());

                    let response = inner.call(request).await?;

                    let response = modify_headers_axum(response, &payment_channel);
                    Ok(response)
                }
                Err(err) => {
                    let mut error_response = Response::new(Body::from(
                        json!({
                            "error": err.to_string()
                        })
                        .to_string(),
                    ));
                    *error_response.status_mut() = err;
                    Ok(error_response)
                }
            }
        })
    }
}

async fn check_payment_channel_middleware(
    state: ChannelState,
    payment_amount: U256, // defined by the developer creating the API, and should match with what user agreed with in the signed request
    request: Request<axum::body::Body>,
) -> Result<(Request<Body>, PaymentChannel, ChannelState), StatusCode> {
    println!("\n=== auth_middleware ===");
    println!(" === new request ===");

    // Get request body
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed: Body decode");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let signed_request = parse_headers(&parts.headers, body_bytes.to_vec(), payment_amount).await?;

    // Check for rate limiting
    state
        .check_rate_limit(signed_request.payment_channel.sender)
        .await?;

    let body_bytes = signed_request.body_bytes.clone();
    // Validate the headers against the payment channel state and return the response
    let updated_channel = verify_and_update_channel(&state, signed_request).await?;

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        updated_channel,
        state,
    ))
}

//* ONE TIME PAYENT MIDDLEWARE LOGIC */
#[derive(Clone)]
pub struct OneTimePaymentMiddlewareState {
    pub config: OneTimePaymentConfig,
}

pub async fn onetime_payment_auth_middleware(
    State(state): State<OneTimePaymentMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("\n=== onetime_tx_auth_middleware ===");
    println!("=== new request ===");

    let signed_payment_tx = parse_tx_headers_axum(&request.headers().clone()).await?;

    let verify = verify_tx(signed_payment_tx, state.config).await?;

    if verify {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
