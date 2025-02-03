use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

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
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status: StatusCode = self.clone().into(); // Convert AuthError to StatusCode
        let body = Json(json!({ "error": self.to_string() })); // Use the error message
        (status, body).into_response()
    }
}
