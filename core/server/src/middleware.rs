use std::{future::Future, pin::Pin, time::Instant};

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
use tracing::info;

use crate::{
    benchmark::log_benchmark,
    channel::ChannelState,
    types::{tx::StreamsConfig, OneTimePaymentConfig, PaymentChannel},
    utils::{
        headers::{parse_stream_headers, parse_tx_headers_axum},
        modify_headers_axum, parse_headers,
    },
    verify::{verify_and_update_channel, verify_stream, verify_tx},
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

#[tracing::instrument(skip(state, payment_amount, request))]
async fn check_payment_channel_middleware(
    state: ChannelState,
    payment_amount: U256, // defined by the developer creating the API, and should match with what user agreed with in the signed request
    request: Request<axum::body::Body>,
) -> Result<(Request<Body>, PaymentChannel, ChannelState), StatusCode> {
    let start = Instant::now();
    println!("\n=== auth_middleware ===");
    println!(" === new request ===");

    let parse_start = Instant::now();
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
    info!("Parse headers time: {:?}", parse_start.elapsed());
    log_benchmark(
        "Parse headers",
        parse_start.elapsed().as_millis(),
        "channel",
    );

    let rate_limit_start = Instant::now();
    // Check for rate limiting
    state
        .check_rate_limit(signed_request.payment_channel.sender)
        .await?;
    info!("Rate limit time: {:?}", rate_limit_start.elapsed());
    log_benchmark(
        "Rate limit",
        rate_limit_start.elapsed().as_millis(),
        "channel",
    );

    let body_bytes = signed_request.body_bytes.clone();
    // Validate the headers against the payment channel state and return the response

    let verify_start = Instant::now();
    let updated_channel = verify_and_update_channel(&state, signed_request).await?;
    info!("Verify time: {:?}", verify_start.elapsed());
    log_benchmark("Verify", verify_start.elapsed().as_millis(), "channel");

    info!("Middleware time: {:?}", start.elapsed());
    log_benchmark("Middleware", start.elapsed().as_millis(), "channel");

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        updated_channel,
        state,
    ))
}

//* ONE TIME PAYENT MIDDLEWARE LOGIC */
#[derive(Clone, Debug)]
pub struct OneTimePaymentMiddlewareState {
    pub config: OneTimePaymentConfig,
}

#[tracing::instrument(skip(state, request, next))]
pub async fn onetime_payment_auth_middleware(
    State(state): State<OneTimePaymentMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    println!("\n=== onetime_tx_auth_middleware ===");
    println!("=== new request ===");

    let parse_start = Instant::now();
    let signed_payment_tx = parse_tx_headers_axum(&request.headers().clone()).await?;
    info!("Parse headers time: {:?}", parse_start.elapsed());
    log_benchmark(
        "Parse headers",
        parse_start.elapsed().as_millis(),
        "onetime",
    );

    let verify_start = Instant::now();
    let verify = verify_tx(signed_payment_tx, state.config).await?;
    info!("Verify time: {:?}", verify_start.elapsed());
    log_benchmark("Verify", verify_start.elapsed().as_millis(), "onetime");

    info!("One time Middleware time: {:?}", start.elapsed());
    log_benchmark(
        "One time Middleware",
        start.elapsed().as_millis(),
        "onetime",
    );

    if verify {
        println!("Verified");
        println!("=== end middleware check ===");
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// * SUPERFLUID STREAMS MIDDLEWARE LOGIC */
#[derive(Clone, Debug)]
pub struct SuperfluidStreamsMiddlewareState {
    pub config: StreamsConfig,
}

#[tracing::instrument(skip(state, request, next))]
pub async fn superfluid_streams_auth_middleware(
    State(state): State<SuperfluidStreamsMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    println!("\n=== superfluid_streams_auth_middleware ===");
    println!("=== new request ===");

    let parse_state = Instant::now();
    let signed_stream = parse_stream_headers(&request.headers().clone()).await?;
    info!("Parse headers time: {:?}", parse_state.elapsed());
    log_benchmark(
        "Parse headers",
        parse_state.elapsed().as_millis(),
        "streams",
    );

    let verify_state = Instant::now();
    let verify = verify_stream(signed_stream, state.config).await?;
    info!("Verify time: {:?}", verify_state.elapsed());
    log_benchmark("Verify", verify_state.elapsed().as_millis(), "streams");

    info!("Middleware time: {:?}", start.elapsed());
    log_benchmark("Middleware", start.elapsed().as_millis(), "streams");

    if verify {
        println!("Verified");
        println!("=== end middleware check ===");
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
