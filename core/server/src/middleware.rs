use std::{future::Future, pin::Pin};

use alloy::primitives::U256;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use serde_json::json;
use tower::{Layer, Service};

use crate::{
    channel::ChannelState,
    types::PaymentChannel,
    utils::{modify_headers_axum, parse_headers_axum},
    verify::verify_and_update_channel,
};

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PipegateMiddlewareLayer {
    state: ChannelState,
    payment_amount: U256,
}

impl PipegateMiddlewareLayer {
    pub fn new(state: ChannelState, payment_amount: U256) -> Self {
        Self {
            state,
            payment_amount,
        }
    }
}

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

impl<S> Service<Request<Body>> for PipegateMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    // #[cfg(target_arch = "wasm32")]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    #[cfg(not(target_arch = "wasm32"))]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let payment_amount = self.payment_amount;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            match auth_middleware(state, payment_amount, req).await {
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

pub async fn auth_middleware(
    state: ChannelState,
    payment_amount: U256, // defined by the developer creating the API, and should match with what user agreed with in the signed request
    request: Request<axum::body::Body>,
    // next: Next<B>,
) -> Result<(Request<Body>, PaymentChannel, ChannelState), StatusCode> {
    println!("\n=== auth_middleware ===");
    println!(" === new request ===");

    let (signed_request, parts) = parse_headers_axum(request, payment_amount).await?;
    let body_bytes = signed_request.body_bytes.clone();

    // Check for rate limiting
    state
        .check_rate_limit(signed_request.payment_channel.sender)
        .await?;

    // Validate the headers against the payment channel state and return the response
    let updated_channel = verify_and_update_channel(&state, signed_request).await?;

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        updated_channel,
        state,
    ))
}
