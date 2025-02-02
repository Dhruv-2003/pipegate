pub mod types;
pub mod utils;
pub mod verify;

use std::{future::Future, pin::Pin};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use tower::{Layer, Service};

use types::StreamsConfig;

use utils::parse_stream_headers;
use verify::verify_stream;

use crate::error::AuthError;

// * SUPERFLUID STREAMS MIDDLEWARE LOGIC */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct StreamMiddlewareLayer {
    pub config: StreamsConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl StreamMiddlewareLayer {
    pub fn new(config: StreamsConfig) -> Self {
        Self { config }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for StreamMiddlewareLayer {
    type Service = StreamMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        StreamMiddleware {
            config: self.config.clone(),
            inner: service,
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct StreamMiddleware<S> {
    inner: S,
    config: StreamsConfig,
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

        // #[cfg(not(target_arch = "wasm32"))]
        Box::pin(async move {
            println!("\n=== superfluid_streams_auth_middleware ===");
            println!("=== new request ===");

            let signed_stream = match parse_stream_headers(&request.headers().clone()).await {
                Ok(tx) => tx,
                Err(e) => return Ok(e.into_response()),
            };

            let verify = match verify_stream(signed_stream, config).await {
                Ok(v) => v,
                Err(e) => return Ok(e.into_response()),
            };

            if verify {
                println!("Verified");
                println!("=== end middleware check ===");

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
