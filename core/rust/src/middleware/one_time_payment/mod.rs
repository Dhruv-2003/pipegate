pub mod state;
pub mod types;
pub mod utils;
pub mod verify;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};

use state::OneTimePaymentState;
use types::OneTimePaymentConfig;
use utils::parse_tx_headers;
use verify::verify_tx;

use crate::{error::AuthError, middleware::utils::get_current_time};

//* ONE TIME PAYMENT MIDDLEWARE (Deprecated standalone in 0.6.0 in favor of unified PaymentsLayer) */
#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
#[deprecated(
    since = "0.6.0",
    note = "Use middleware::PaymentsLayer (unified PipegateMiddlewareLayer alias)"
)]
pub struct OnetimePaymentMiddlewareLayer {
    pub config: OneTimePaymentConfig,
    pub state: OneTimePaymentState,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(deprecated)]
impl OnetimePaymentMiddlewareLayer {
    pub fn new(config: OneTimePaymentConfig, state: OneTimePaymentState) -> Self {
        Self { config, state }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(deprecated)]
impl<S> Layer<S> for OnetimePaymentMiddlewareLayer {
    type Service = OnetimePaymentMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        OnetimePaymentMiddleware {
            config: self.config.clone(),
            state: self.state.clone(),
            inner: service,
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
#[deprecated(
    since = "0.6.0",
    note = "Use middleware::Payments<S> (unified PipegateMiddleware alias)"
)]
pub struct OnetimePaymentMiddleware<S> {
    inner: S,
    config: OneTimePaymentConfig,
    state: OneTimePaymentState,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(deprecated)]
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
        let state = self.state.clone();
        let mut inner = self.inner.clone();

        // #[cfg(not(target_arch = "wasm32"))]
        Box::pin(async move {
            println!("\n=== onetime_tx_auth_middleware ===");
            println!("=== new request ===");

            let signed_payment_tx = match parse_tx_headers(&request.headers().clone()).await {
                Ok(tx) => tx,
                Err(e) => return Ok(e.into_response()),
            };

            let tx_hash = signed_payment_tx.tx_hash;
            let current_time = get_current_time();

            // Check if payment exists in state
            if let Some(_) = state.get(tx_hash).await {
                println!("Found existing payment in state");

                // Use custom period_ttl_sec if set in config, otherwise fallback to hardcoded values
                let custom_session_ttl = config.period_ttl_sec;

                // Check if the payment is still valid for redemption
                if state
                    .is_valid_for_redemption_with_period(
                        tx_hash,
                        current_time,
                        None,
                        custom_session_ttl,
                    )
                    .await
                {
                    println!("Payment is valid for redemption");

                    // Increment redemption count
                    if let Some(_) = state.increment_redemptions(tx_hash).await {}

                    println!("=== end middleware check ===");
                    inner.call(request).await
                } else {
                    println!("Payment is no longer valid for redemption");
                    Ok(AuthError::InvalidTransaction(
                        "Payment session expired or max redemptions reached".to_string(),
                    )
                    .into_response())
                }
            } else {
                println!("New payment - verifying transaction");

                // First time seeing this payment, verify the transaction
                let (new_payment, verify) = match verify_tx(signed_payment_tx.clone(), config).await
                {
                    Ok(v) => v,
                    Err(e) => return Ok(e.into_response()),
                };

                if verify {
                    println!("Transaction verified successfully");

                    state.set(tx_hash, new_payment).await;
                    println!("Added new payment to state");

                    state.increment_redemptions(tx_hash).await;
                    println!("=== end middleware check ===");

                    inner.call(request).await
                } else {
                    println!("Transaction verification failed");
                    Ok(
                        AuthError::InvalidTransaction("Authentication failed".to_string())
                            .into_response(),
                    )
                }
            }
        })
    }
}

//* ONE TIME PAYMENT MIDDLEWARE LOGIC */
#[derive(Clone)]
pub struct OneTimePaymentFnMiddlewareState {
    pub config: OneTimePaymentConfig,
    pub payment_state: OneTimePaymentState,
}

pub async fn onetime_payment_auth_fn_middleware(
    State(state): State<OneTimePaymentFnMiddlewareState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("\n=== onetime_tx_auth_middleware ===");
    println!("=== new request ===");

    let signed_payment_tx = match parse_tx_headers(&request.headers().clone()).await {
        Ok(tx) => tx,
        Err(e) => return Ok(e.into_response()),
    };

    let tx_hash = signed_payment_tx.tx_hash;
    let current_time = get_current_time();

    // Check if payment exists in state
    if let Some(existing_payment) = state.payment_state.get(tx_hash).await {
        println!("Found existing payment in state");

        // Use custom period_ttl_sec if set in config, otherwise fallback to hardcoded values
        let custom_session_ttl = state.config.period_ttl_sec;

        // Check if the payment is still valid for redemption
        if state
            .payment_state
            .is_valid_for_redemption_with_period(tx_hash, current_time, None, custom_session_ttl)
            .await
        {
            println!("Payment is valid for redemption");

            // Set first redeemed timestamp if this is the first time
            if existing_payment.first_reedemed == 0 {
                state
                    .payment_state
                    .set_first_redeemed(tx_hash, current_time)
                    .await;
                println!("Set first redemption timestamp");
            }

            // Increment redemption count
            if let Some(new_count) = state.payment_state.increment_redemptions(tx_hash).await {
                println!("Incremented redemptions to: {}", new_count);
            }

            println!("=== end middleware check ===");
            Ok(next.run(request).await)
        } else {
            println!("Payment is no longer valid for redemption");
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        println!("New payment - verifying transaction");

        // First time seeing this payment, verify the transaction
        let (new_payment, verify) = match verify_tx(signed_payment_tx.clone(), state.config).await {
            Ok(v) => v,
            Err(e) => return Ok(e.into_response()),
        };

        if verify {
            println!("Transaction verified successfully");

            state.payment_state.set(tx_hash, new_payment).await;
            println!("Added new payment to state");

            state.payment_state.increment_redemptions(tx_hash).await;
            println!("=== end middleware check ===");

            Ok(next.run(request).await)
        } else {
            println!("Transaction verification failed");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
