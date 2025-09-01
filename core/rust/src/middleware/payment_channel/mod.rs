pub mod channel;
pub mod types;
pub mod utils;
pub mod verify;

mod extractors;

use std::{future::Future, pin::Pin};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use tower::{Layer, Service};

use channel::ChannelState;

use types::PaymentChannelConfig;
use utils::{modify_headers_axum, parse_headers};
use verify::verify_and_update_channel;

use crate::error::AuthError;

//* PAYMENT CHANNEL MIDDLEWARE LOGIC */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PaymentChannelMiddlewareLayer {
    pub state: ChannelState,
    pub config: PaymentChannelConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl PaymentChannelMiddlewareLayer {
    pub fn new(state: ChannelState, config: PaymentChannelConfig) -> Self {
        Self { state, config }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for PaymentChannelMiddlewareLayer {
    type Service = PaymentChannelMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        PaymentChannelMiddleware {
            state: self.state.clone(),
            config: self.config.clone(),
            inner: service,
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PaymentChannelMiddleware<S> {
    inner: S,
    state: ChannelState,
    config: PaymentChannelConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Service<Request<Body>> for PaymentChannelMiddleware<S>
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
        let state = self.state.clone();
        let config = self.config.clone();
        let payment_amount = self.config.clone().amount;
        let mut inner = self.inner.clone();

        // #[cfg(not(target_arch = "wasm32"))]
        Box::pin(async move {
            println!("\n=== auth_middleware ===");
            println!(" === new request ===");

            // Get request body
            let (parts, body) = request.into_parts();
            let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    println!("Failed: Body decode");
                    return Ok(
                        AuthError::InvalidRequest("Failed to decode body".to_string())
                            .into_response(),
                    );
                }
            };

            let signed_request =
                match parse_headers(&parts.headers, body_bytes.to_vec(), payment_amount).await {
                    Ok(signed_request) => signed_request,
                    Err(e) => {
                        println!("Failed: Parse headers");
                        return Ok(e.into_response());
                    }
                };

            let body_bytes = signed_request.body_bytes.clone();
            // Validate the headers against the payment channel state and return the response
            let (updated_channel, verify) =
                match verify_and_update_channel(&state, &config, signed_request).await {
                    Ok((updated_channel, verify)) => (updated_channel, verify),
                    Err(e) => {
                        println!("Failed: Verify and update channel");
                        return Ok(e.into_response());
                    }
                };

            state
                .channels
                .write()
                .await
                .insert(updated_channel.channel_id, updated_channel.clone());

            if verify {
                println!("Verified");
                println!("=== end middleware check ===");

                let request = Request::from_parts(parts, Body::from(body_bytes));
                let response = inner.call(request).await?;
                let response = modify_headers_axum(response, &updated_channel);
                Ok(response)
            } else {
                Ok(AuthError::InternalError.into_response())
            }
        })
    }
}

#[derive(Clone)]
pub struct PaymentChannelFnMiddlewareState {
    state: ChannelState,
    config: PaymentChannelConfig,
}

pub async fn payment_channel_auth_fn_middleware(
    State(state): State<PaymentChannelFnMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("\n=== auth_middleware ===");
    println!(" === new request ===");

    // Get request body
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed: Body decode");
            return Ok(
                AuthError::InvalidRequest("Failed to decode body".to_string()).into_response(),
            );
        }
    };

    let signed_request =
        match parse_headers(&parts.headers, body_bytes.to_vec(), state.config.amount).await {
            Ok(signed_request) => signed_request,
            Err(e) => {
                println!("Failed: Parse headers");
                return Ok(e.into_response());
            }
        };

    let body_bytes = signed_request.body_bytes.clone();
    // Validate the headers against the payment channel state and return the response
    let (updated_channel, verify) =
        match verify_and_update_channel(&state.state, &state.config, signed_request).await {
            Ok((updated_channel, verify)) => (updated_channel, verify),
            Err(e) => {
                println!("Failed: Verify and update channel");
                return Ok(e.into_response());
            }
        };

    state
        .state
        .channels
        .write()
        .await
        .insert(updated_channel.channel_id, updated_channel.clone());

    let request = Request::from_parts(parts, Body::from(body_bytes));

    if verify {
        println!("Verified");
        println!("=== end middleware check ===");

        let response = next.run(request).await;
        let response = modify_headers_axum(response, &updated_channel);
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
