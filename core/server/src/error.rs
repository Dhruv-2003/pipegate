use alloy::rpc::types::error;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing required headers")]
    MissingHeaders,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient payment channel balance")]
    InsufficientBalance,
    #[error("Payment channel expired")]
    Expired,
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
}

impl From<AuthError> for StatusCode {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::MissingHeaders => StatusCode::BAD_REQUEST,
            AuthError::InvalidSignature => StatusCode::UNAUTHORIZED,
            AuthError::InsufficientBalance => StatusCode::PAYMENT_REQUIRED,
            AuthError::Expired => StatusCode::REQUEST_TIMEOUT,
            AuthError::InvalidChannel => StatusCode::BAD_REQUEST,
            AuthError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AuthError::ChannelNotFound => StatusCode::NOT_FOUND,
            AuthError::ContractError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::NetworkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidConfig => StatusCode::BAD_REQUEST,
        }
    }
}
