pub mod listener;
pub mod state;
pub mod types;
pub mod utils;
pub mod verify;

use std::{future::Future, pin::Pin, time::SystemTime};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub use listener::StreamListner;

use state::StreamState;
use tower::{Layer, Service};

pub use types::{Stream, StreamsConfig};

use utils::parse_stream_headers;
use verify::verify_stream;

use crate::error::AuthError;

// * SUPERFLUID STREAMS MIDDLEWARE LOGIC */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct StreamMiddlewareLayer {
    pub config: StreamsConfig,
    pub state: StreamState,
}

#[cfg(not(target_arch = "wasm32"))]
impl StreamMiddlewareLayer {
    pub fn new(config: StreamsConfig, state: StreamState) -> Self {
        Self { config, state }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for StreamMiddlewareLayer {
    type Service = StreamMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        StreamMiddleware {
            inner: service,
            config: self.config.clone(),
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct StreamMiddleware<S> {
    inner: S,
    config: StreamsConfig,
    state: StreamState,
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Service<Request<Body>> for StreamMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        // #[cfg(not(target_arch = "wasm32"))]
        Box::pin(async move {
            println!("\n=== superfluid_streams_auth_middleware ===");
            println!("=== new request ===");

            let signed_stream = match parse_stream_headers(&request.headers().clone()).await {
                Ok(tx) => tx,
                Err(e) => return Ok(e.into_response()),
            };

            // Check if stream was already verified earlier
            if let Some(stream) = state.get(signed_stream.sender).await {
                if stream.last_verified > 0 {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if timestamp - stream.last_verified < config.cache_time {
                        println!("Stream already verified, in Cache!");
                        println!("=== end middleware check ===");

                        return inner.call(request).await;
                    }
                }
            }

            let verify = match verify_stream(signed_stream.clone(), config.clone()).await {
                Ok(v) => v,
                Err(e) => return Ok(e.into_response()),
            };

            if verify {
                println!("Verified");
                println!("=== end middleware check ===");

                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Add the verified stream to the record
                let stream = Stream {
                    sender: signed_stream.sender,
                    recipient: config.recipient,
                    token_address: config.token_address,
                    flow_rate: config.amount,
                    last_verified: timestamp,
                };

                state.set(signed_stream.sender, stream).await;

                // start the inner subscribe/listener service for events of this sender

                inner.call(request).await
            } else {
                Ok(
                    AuthError::InvalidTransaction("Authentication failed".to_string())
                        .into_response(),
                )
            }
        })
    }
}

#[derive(Clone)]
pub struct SuperfluidStreamsFnMiddlewareState {
    pub config: StreamsConfig,
    pub state: StreamState,
}

pub async fn superfluid_streams_auth_fn_middleware(
    State(state): State<SuperfluidStreamsFnMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("\n=== superfluid_streams_auth_middleware ===");
    println!("=== new request ===");

    let signed_stream = match parse_stream_headers(&request.headers().clone()).await {
        Ok(tx) => tx,
        Err(e) => return Ok(e.into_response()),
    };

    let verify = match verify_stream(signed_stream, state.config).await {
        Ok(v) => v,
        Err(e) => return Ok(e.into_response()),
    };

    if verify {
        println!("Verified");
        println!("=== end middleware check ===");
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
