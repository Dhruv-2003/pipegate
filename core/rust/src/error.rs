use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::middleware::{
    one_time_payment::types::{ABS_WINDOW_SEC, MAX_REDEMPTIONS, SESSION_TTL_SEC},
    types::{PaymentRequiredAccept, PaymentRequiredResponse},
};

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("Missing required headers")]
    MissingHeaders,
    #[error("Invalid headers")]
    InvalidHeaders,
    #[error("Invalid timestamp")]
    TimestampError,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient payment channel balance")]
    InsufficientBalance,
    #[error("Payment channel expired")]
    Expired,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Invalid payment channel")]
    InvalidChannel,
    #[error("Channel not found")]
    ChannelNotFound,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Contract interaction failed: {0}")]
    ContractError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid network configuration")]
    InvalidConfig,
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Invalid request : {0}")]
    InvalidRequest(String),
    #[error("Internal Error")]
    InternalError,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Invalid Transaction : {0}")]
    InvalidTransaction(String),
    #[error("Invalid Stream : {0}")]
    InvalidStream(String),
    #[error("Invalid sender")]
    InvalidSender,
    #[error("Payment scheme not accepted")]
    SchemeNotAccepted,
}

impl From<AuthError> for StatusCode {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::MissingHeaders => StatusCode::BAD_REQUEST,
            AuthError::InvalidHeaders => StatusCode::BAD_REQUEST,
            AuthError::TimestampError => StatusCode::REQUEST_TIMEOUT,
            AuthError::InvalidSignature => StatusCode::UNAUTHORIZED,
            AuthError::InsufficientBalance => StatusCode::PAYMENT_REQUIRED,
            AuthError::Expired => StatusCode::UNAUTHORIZED,
            AuthError::InvalidNonce => StatusCode::BAD_REQUEST,
            AuthError::InvalidChannel => StatusCode::BAD_REQUEST,
            AuthError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AuthError::ChannelNotFound => StatusCode::NOT_FOUND,
            AuthError::ContractError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::NetworkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidConfig => StatusCode::BAD_REQUEST,
            AuthError::InvalidMessage => StatusCode::BAD_REQUEST,
            AuthError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            AuthError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::TransactionNotFound => StatusCode::BAD_REQUEST,
            AuthError::InvalidTransaction(_) => StatusCode::BAD_REQUEST,
            AuthError::InvalidStream(_) => StatusCode::BAD_REQUEST,
            AuthError::InvalidSender => StatusCode::BAD_REQUEST,
            AuthError::SchemeNotAccepted => StatusCode::FORBIDDEN,
        }
    }
}

impl AuthError {
    pub fn into_payment_required_response(self, accepts: Vec<PaymentRequiredAccept>) -> Response {
        let payment_required = PaymentRequiredResponse {
            x402_version: 1,
            accepts,
            error: self.to_string(),
        };

        let status = StatusCode::PAYMENT_REQUIRED;
        (status, Json(payment_required)).into_response()
    }

    pub fn into_x402_response(
        self,
        config: &crate::middleware::types::MiddlewareConfig,
        resource: &str,
    ) -> Response {
        let accepts: Vec<PaymentRequiredAccept> = config
            .accepts
            .iter()
            .map(|scheme_config| {
                let extra = match scheme_config.scheme {
                    crate::middleware::types::Scheme::OneTimePayments => Some(serde_json::json!({
                        "absWindowSeconds": ABS_WINDOW_SEC,
                        "sessionTTLSeconds": SESSION_TTL_SEC,
                        "maxRedemptions": MAX_REDEMPTIONS
                    })),
                    crate::middleware::types::Scheme::SuperfluidStreams => None,
                    crate::middleware::types::Scheme::PaymentChannels => {
                        Some(serde_json::json!("paymentChannelState"))
                    }
                };

                PaymentRequiredAccept {
                    scheme: scheme_config.scheme.to_string().to_string(),
                    network: scheme_config.chain_name.clone(),
                    amount: scheme_config.amount.clone(),
                    pay_to: scheme_config.recipient.to_string(),
                    asset: scheme_config.token_address.to_string(),
                    resource: resource.to_string(),
                    description: Some(format!("Access to {} resource", resource)),
                    max_timeout_seconds: Some(300), // 5 minutes default
                    extra,
                }
            })
            .collect();

        self.into_payment_required_response(accepts)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        // let status: StatusCode = self.clone().into(); // Convert AuthError to StatusCode

        // Returning the 402 Payment Required as all the errors are related to payment/auth middleware here, keeping this consistent
        let status = StatusCode::PAYMENT_REQUIRED;
        let body = Json(json!({ "error": self.to_string() })); // Use the error message
        (status, body).into_response()
    }
}
