pub mod one_time_payment;
pub mod payment_channel;
pub mod stream_payment;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod state;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod types;

mod utils;

use alloy::primitives::{aliases::I96, utils::parse_units, Address};
use axum::{body::Body, http::Request, response::Response};
use std::{future::Future, pin::Pin, str::FromStr};
use tower::{Layer, Service};

#[cfg(not(target_arch = "wasm32"))]
pub use state::MiddlewareState;
#[cfg(not(target_arch = "wasm32"))]
pub use types::{MiddlewareConfig, Scheme, SchemeConfig};

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    error::AuthError,
    middleware::{
        one_time_payment::{types::OneTimePaymentConfig, verify::verify_tx},
        payment_channel::{
            types::{PaymentChannel, PaymentChannelConfig},
            utils::modify_headers_axum,
            verify::verify_and_update_channel,
        },
        stream_payment::{
            types::{StreamsConfig, CFA_V1_FORWARDER_ADDRESS},
            verify::verify_stream,
        },
        types::{PaymentHeader, PaymentPayload},
        utils::{
            get_current_time, parse_channel_payload, parse_onetime_payload, parse_stream_payload,
        },
    },
};

// Preferred public naming aliases for unified middleware (introduced in 0.6.0)
#[doc = "Alias for the unified payments middleware state (preferred external name)"]
#[cfg(not(target_arch = "wasm32"))]
pub type PaymentsState = MiddlewareState;

#[doc = "Alias for the unified payments middleware config (preferred external name)"]
#[cfg(not(target_arch = "wasm32"))]
pub type PaymentsConfig = MiddlewareConfig;

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PipegateMiddlewareLayer {
    pub state: MiddlewareState,
    pub config: MiddlewareConfig,
}

/// Preferred alias: use `PaymentsLayer` in new code (added in 0.6.0)
#[doc = "Unified payments middleware layer handling all supported schemes. Prefer this name over PipegateMiddlewareLayer in new code."]
#[cfg(not(target_arch = "wasm32"))]
pub type PaymentsLayer = PipegateMiddlewareLayer;

#[cfg(not(target_arch = "wasm32"))]
impl PipegateMiddlewareLayer {
    pub fn new(state: MiddlewareState, config: MiddlewareConfig) -> Self {
        Self { state, config }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<S> Layer<S> for PipegateMiddlewareLayer {
    type Service = PipegateMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        PipegateMiddleware {
            inner: service,
            state: self.state.clone(),
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PipegateMiddleware<S> {
    inner: S,
    state: MiddlewareState,
    config: MiddlewareConfig,
}

/// Preferred alias: use `Payments` in new code (added in 0.6.0)
#[doc = "Unified payments middleware service wrapper. Prefer this alias over PipegateMiddleware in new code."]
#[cfg(not(target_arch = "wasm32"))]
pub type Payments<S> = PipegateMiddleware<S>;

#[cfg(not(target_arch = "wasm32"))]
impl<S> Service<Request<Body>> for PipegateMiddleware<S>
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
        let mut state = self.state.clone();
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let resource = request.uri().path();

            // Helper function to create x402 responses
            let create_x402_response =
                |error: AuthError, payment_channel: Option<PaymentChannel>| {
                    error.into_x402_response(&config, resource, payment_channel)
                };

            // 1. Check for X-Payment headers -> PaymentRequiredHeader
            let headers = request.headers();

            let payment_header = match headers.get("X-Payment") {
                Some(h) => h,
                None => {
                    return Ok(create_x402_response(AuthError::MissingHeaders, None));
                }
            };

            let payment_json = match payment_header.to_str().map_err(|_| {
                AuthError::InvalidHeaders(
                    "X-Payment header contains invalid UTF-8 characters".to_string(),
                )
            }) {
                Ok(h) => h,
                Err(e) => return Ok(create_x402_response(e, None)),
            };

            let payment: PaymentHeader = match serde_json::from_str(payment_json).map_err(|e| {
                AuthError::InvalidHeaders(format!("Failed to parse X-Payment JSON: {}", e))
            }) {
                Ok(p) => p,
                Err(e) => return Ok(create_x402_response(e, None)),
            };

            if !payment.validate_payload_for_scheme() {
                return Ok(create_x402_response(
                    AuthError::InvalidHeaders(format!(
                        "Payload type mismatch: scheme '{}' does not match payload type",
                        payment.scheme
                    )),
                    None,
                ));
            }

            // 2. Route to the correct child middleware logic based on scheme
            let verification_result: Result<Option<PaymentChannel>, AuthError> = match payment
                .get_scheme_enum()
            {
                Some(Scheme::OneTimePayments) => {
                    if let PaymentPayload::OneTime(payload) = payment.payload {
                        // 3. Verify if the user accepts this scheme even in the config

                        let scheme_config = match config.get_scheme_config(Scheme::OneTimePayments)
                        {
                            Some(c) => c,
                            None => {
                                return Ok(create_x402_response(AuthError::SchemeNotAccepted, None))
                            }
                        };

                        let amount = match parse_units(
                            &scheme_config.amount,
                            scheme_config.decimals.unwrap_or(18),
                        ) {
                            Ok(a) => a.get_absolute(),
                            Err(_) => {
                                return Ok(create_x402_response(AuthError::InternalError, None));
                            }
                        };

                        let onetime_config = OneTimePaymentConfig {
                            rpc_url: scheme_config.network_rpc_url.clone(),
                            token_address: scheme_config.token_address,
                            recipient: scheme_config.recipient,
                            amount: amount,
                            period_ttl_sec: None,
                        };

                        let payment = match parse_onetime_payload(&payload).await {
                            Ok(p) => p,
                            Err(e) => return Ok(create_x402_response(e, None)),
                        };

                        if state.one_time_payment_state.read().await.is_none() {
                            println!("Initialising one-time payment state");
                            state = state.with_one_time_payment_state().await;
                        }

                        // We need to drop the read guard and use the state directly since OneTimePaymentState methods handle their own locking
                        let one_time_payment_state = {
                            let guard = state.one_time_payment_state.read().await;
                            guard.as_ref().unwrap().clone()
                        };
                        let current_time = get_current_time();

                        if let Some(_) = one_time_payment_state.get(payment.tx_hash).await {
                            println!("Found existing payment in state");

                            // Check if the payment is still valid for redemption
                            if one_time_payment_state
                                .is_valid_for_redemption_with_period(
                                    payment.tx_hash,
                                    current_time,
                                    None,
                                    None,
                                )
                                .await
                            {
                                println!("Payment is valid for redemption");

                                one_time_payment_state
                                    .increment_redemptions(payment.tx_hash)
                                    .await;
                                println!("=== end middleware check ===");
                            } else {
                                println!("Payment is no longer valid for redemption");
                                return Ok(create_x402_response(
                                    AuthError::InvalidTransaction(
                                        "Payment session expired or max redemptions reached"
                                            .to_string(),
                                    ),
                                    None,
                                ));
                            }
                        } else {
                            println!("New payment - verifying transaction");

                            // First time seeing this payment, verify the transaction
                            let (new_payment, verify) =
                                match verify_tx(payment.clone(), onetime_config).await {
                                    Ok(v) => v,
                                    Err(e) => return Ok(create_x402_response(e, None)),
                                };

                            if !verify {
                                println!("Transaction verification failed");
                                return Ok(create_x402_response(
                                    AuthError::InvalidTransaction(
                                        "Authentication failed".to_string(),
                                    ),
                                    None,
                                ));
                            }

                            println!("Transaction verified successfully");

                            one_time_payment_state
                                .set(payment.clone().tx_hash, new_payment)
                                .await;
                            println!("Added new payment to state");

                            one_time_payment_state
                                .increment_redemptions(payment.clone().tx_hash)
                                .await;
                        }
                        Ok(None)
                    } else {
                        Err(AuthError::InvalidHeaders(
                            "Expected OneTime payload for one-time payment scheme".to_string(),
                        ))
                    }
                }
                Some(Scheme::SuperfluidStreams) => {
                    if let PaymentPayload::Stream(payload) = payment.payload {
                        let scheme_config = match config
                            .get_scheme_config(Scheme::SuperfluidStreams)
                        {
                            Some(c) => c,
                            None => {
                                return Ok(create_x402_response(AuthError::SchemeNotAccepted, None))
                            }
                        };

                        let signed_stream = match parse_stream_payload(&payload).await {
                            Ok(s) => s,
                            Err(e) => return Ok(create_x402_response(e, None)),
                        };

                        let flow_rate = {
                            let monthly_amount = match scheme_config.amount.parse::<f64>() {
                                Ok(amount) => amount,
                                Err(_) => {
                                    return Ok(create_x402_response(AuthError::InternalError, None))
                                }
                            };

                            let decimals = scheme_config.decimals.unwrap_or(18);
                            let amount_with_decimals =
                                monthly_amount * (10_f64.powi(decimals as i32));

                            let flow_rate_per_second =
                                amount_with_decimals / ((365.0 / 12.0) * 24.0 * 60.0 * 60.0);

                            let flow_rate_i128 = flow_rate_per_second as i128;
                            I96::try_from(flow_rate_i128).unwrap_or(I96::ZERO)
                        };

                        let streams_config = StreamsConfig {
                            rpc_url: scheme_config.network_rpc_url.clone(),
                            cfa_forwarder: Address::from_str(CFA_V1_FORWARDER_ADDRESS).unwrap(),
                            token_address: scheme_config.token_address,
                            recipient: scheme_config.recipient,
                            amount: flow_rate,
                            cache_time: 900,
                        };

                        if state.stream_state.read().await.is_none() {
                            println!("Initialising stream state");
                            state = state.with_stream_state().await;
                            #[allow(unused_must_use)]
                            state.start_stream_listener(scheme_config.chain_id, &streams_config);
                        }

                        // We need to drop the read guard and use the state directly since StreamState methods handle their own locking
                        let stream_state = {
                            let guard = state.stream_state.read().await;
                            guard.as_ref().unwrap().clone()
                        };
                        let current_time = get_current_time();

                        if let Some(stream) = stream_state.get(signed_stream.sender).await {
                            if stream.last_verified > 0
                                && current_time - stream.last_verified < streams_config.cache_time
                            {
                                println!("Stream already verified, in Cache!");
                            } else {
                                let verify = match verify_stream(
                                    signed_stream.clone(),
                                    streams_config.clone(),
                                )
                                .await
                                {
                                    Ok(v) => v,
                                    Err(e) => return Ok(create_x402_response(e, None)),
                                };

                                if !verify {
                                    return Ok(create_x402_response(
                                        AuthError::InvalidTransaction(
                                            "Stream verification failed".to_string(),
                                        ),
                                        None,
                                    ));
                                }

                                let updated_stream =
                                    crate::middleware::stream_payment::types::Stream {
                                        sender: signed_stream.sender,
                                        recipient: streams_config.recipient,
                                        token_address: streams_config.token_address,
                                        flow_rate: streams_config.amount,
                                        last_verified: current_time,
                                    };

                                stream_state.set(signed_stream.sender, updated_stream).await;
                                println!("Stream verified and updated");
                            }
                        } else {
                            let verify =
                                match verify_stream(signed_stream.clone(), streams_config.clone())
                                    .await
                                {
                                    Ok(v) => v,
                                    Err(e) => return Ok(create_x402_response(e, None)),
                                };

                            if !verify {
                                return Ok(create_x402_response(
                                    AuthError::InvalidTransaction(
                                        "Stream verification failed".to_string(),
                                    ),
                                    None,
                                ));
                            }

                            let new_stream = crate::middleware::stream_payment::types::Stream {
                                sender: signed_stream.sender,
                                recipient: streams_config.recipient,
                                token_address: streams_config.token_address,
                                flow_rate: streams_config.amount,
                                last_verified: current_time,
                            };

                            stream_state.set(signed_stream.sender, new_stream).await;
                            println!("New stream verified and added");
                        }

                        Ok(None)
                    } else {
                        Err(AuthError::InvalidHeaders(
                            "Expected Stream payload for stream payment scheme".to_string(),
                        ))
                    }
                }
                Some(Scheme::PaymentChannels) => {
                    if let PaymentPayload::Channel(payload) = payment.payload {
                        let scheme_config = match config.get_scheme_config(Scheme::PaymentChannels)
                        {
                            Some(c) => c,
                            None => {
                                return Ok(create_x402_response(AuthError::SchemeNotAccepted, None))
                            }
                        };

                        let amount = match parse_units(
                            &scheme_config.amount,
                            scheme_config.decimals.unwrap_or(18),
                        ) {
                            Ok(a) => a.get_absolute(),
                            Err(_) => {
                                return Ok(create_x402_response(AuthError::InternalError, None))
                            }
                        };

                        let (signature, message, payment_channel) =
                            match parse_channel_payload(&payload).await {
                                Ok(data) => data,
                                Err(e) => return Ok(create_x402_response(e, None)),
                            };

                        let channel_config = PaymentChannelConfig {
                            rpc_url: scheme_config.network_rpc_url.clone(),
                            token_address: scheme_config.token_address,
                            recipient: scheme_config.recipient,
                            amount,
                        };

                        if state.channel_state.read().await.is_none() {
                            println!("Initialising channel state");
                            state = state.with_channel_state().await;
                        }

                        // We need to drop the read guard and use the state directly since ChannelState methods handle their own locking
                        let channel_state = {
                            let guard = state.channel_state.read().await;
                            guard.as_ref().unwrap().clone()
                        };

                        let existing_channel =
                            channel_state.get_channel(payment_channel.channel_id).await;

                        let signed_request =
                            crate::middleware::payment_channel::types::SignedRequest {
                                message,
                                signature,
                                payment_channel: payment_channel.clone(),
                                payment_amount: amount,
                                body_bytes: Vec::new(), // NOTE: We don't use body_bytes anymore
                                timestamp: payload.timestamp,
                            };

                        let (updated_channel, verify) = match verify_and_update_channel(
                            &channel_state,
                            &channel_config,
                            signed_request,
                        )
                        .await
                        {
                            Ok((channel, verified)) => (channel, verified),
                            Err(e) => return Ok(create_x402_response(e, existing_channel)),
                        };

                        if verify {
                            channel_state
                                .channels
                                .write()
                                .await
                                .insert(updated_channel.channel_id, updated_channel.clone());

                            channel_state
                                .update_latest_signature(updated_channel.channel_id, signature)
                                .await;

                            println!("Channel verified and updated");
                        } else {
                            return Ok(create_x402_response(
                                AuthError::InvalidTransaction(
                                    "Channel verification failed".to_string(),
                                ),
                                existing_channel,
                            ));
                        }
                        Ok(Some(updated_channel.clone()))
                    } else {
                        Err(AuthError::InvalidHeaders(
                            "Expected Channel payload for payment channel scheme".to_string(),
                        ))
                    }
                }
                None => Err(AuthError::InvalidHeaders(
                    "Unknown or unsupported payment scheme".to_string(),
                )),
            };

            println!("=== end middleware check ===");

            // Handle verification result
            match verification_result {
                Ok(payment_channel) => {
                    // Payment verified, proceed with the request
                    let response = inner.call(request).await?;
                    if let Some(updated_channel) = payment_channel {
                        let response = modify_headers_axum(response, &updated_channel);
                        return Ok(response);
                    }

                    Ok(response)
                }
                Err(auth_error) => {
                    // Payment verification failed, return 402 Payment Required with proper x402 format
                    Ok(create_x402_response(auth_error, None))
                }
            }
        })
    }
}
