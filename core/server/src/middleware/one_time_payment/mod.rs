pub mod types;
pub mod utils;
pub mod verify;

#[cfg(not(target_arch = "wasm32"))]
use std::{future::Future, pin::Pin};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use tower::{Layer, Service};

use types::OneTimePaymentConfig;
use utils::parse_tx_headers_axum;
use verify::verify_tx;

use crate::error::AuthError;

//* PAYMENT CHANNEL MIDDLEWARE LOGIC */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct OnetimePaymentMiddlewareLayer {
    pub config: OneTimePaymentConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl OnetimePaymentMiddlewareLayer {
    pub fn new(config: OneTimePaymentConfig) -> Self {
        Self { config }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for OnetimePaymentMiddlewareLayer {
    type Service = OnetimePaymentMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        OnetimePaymentMiddleware {
            config: self.config.clone(),
            inner: service,
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct OnetimePaymentMiddleware<S> {
    inner: S,
    config: OneTimePaymentConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Service<Request<Body>> for OnetimePaymentMiddleware<S>
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
            println!("\n=== onetime_tx_auth_middleware ===");
            println!("=== new request ===");

            let signed_payment_tx = match parse_tx_headers_axum(&request.headers().clone()).await {
                Ok(tx) => tx,
                Err(e) => return Ok(e.into_response()),
            };

            let verify = match verify_tx(signed_payment_tx, config).await {
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

//* ONE TIME PAYENT MIDDLEWARE LOGIC */
#[derive(Clone)]
pub struct OneTimePaymentFnMiddlewareState {
    pub config: OneTimePaymentConfig,
}

pub async fn onetime_payment_auth_fn_middleware(
    State(state): State<OneTimePaymentFnMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("\n=== onetime_tx_auth_middleware ===");
    println!("=== new request ===");

    let signed_payment_tx = match parse_tx_headers_axum(&request.headers().clone()).await {
        Ok(tx) => tx,
        Err(e) => return Ok(e.into_response()),
    };

    let verify = match verify_tx(signed_payment_tx, state.config).await {
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
