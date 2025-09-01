use alloy::{
    hex,
    primitives::{FixedBytes, PrimitiveSignature},
};
use axum::{async_trait, extract::FromRequestParts};
use http::{request::Parts, StatusCode};

use crate::middleware::payment_channel::types::PaymentChannel;

#[allow(dead_code)]
struct PaymentChannelExtractor {
    pub message: Vec<u8>,
    pub signature: PrimitiveSignature,
    pub payment_channel: PaymentChannel,
    pub timestamp: u64,
}

#[async_trait]
impl<S> FromRequestParts<S> for PaymentChannelExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let signature = parts
            .headers
            .get("X-Signature")
            .ok_or((StatusCode::BAD_REQUEST, "X-Signature not found in headers"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Signature couldn't be parsed"))?;

        let message = parts
            .headers
            .get("X-Message")
            .ok_or((StatusCode::BAD_REQUEST, "X-Message not found in headers"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Message couldn't be parsed"))?;

        let payment_data = parts
            .headers
            .get("X-Payment")
            .ok_or((StatusCode::BAD_REQUEST, "X-Payment not found in headers"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Payment couldn't be parsed"))?;

        let timestamp = parts
            .headers
            .get("X-Timestamp")
            .and_then(|t| t.to_str().ok())
            .and_then(|t| t.parse::<u64>().ok())
            .ok_or((StatusCode::BAD_REQUEST, "X-Timestamp not found in headers"))?;

        // Parse signature
        let signature = hex::decode(signature.trim_start_matches("0x"))
            .map_err(|_| {
                println!("Failed: Signature decode");
                (StatusCode::BAD_REQUEST, "Signature couldn't be decoded")
            })
            .and_then(|bytes| {
                PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                    println!("Failed: Signature conversion");
                    (StatusCode::BAD_REQUEST, "Signature couldn't be converted")
                })
            })?;

        // Parse message
        let message = hex::decode(message).map_err(|_| {
            println!("Failed: Message decode");
            (StatusCode::BAD_REQUEST, "Message couldn't be decoded")
        })?;

        // Parse payment channel data
        let payment_channel: PaymentChannel = serde_json::from_str(payment_data).map_err(|e| {
            println!("Failed: Payment data decode - Error {}", e);
            (StatusCode::BAD_REQUEST, "Payment couldn't be decoded")
        })?;

        Ok(Self {
            message,
            signature,
            payment_channel,
            timestamp,
        })
    }
}

#[allow(dead_code)]
struct OneTimePaymentExtractor {
    pub signature: PrimitiveSignature,
    pub tx_hash: FixedBytes<32>,
}

#[async_trait]
impl<S> FromRequestParts<S> for OneTimePaymentExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let signature = parts
            .headers
            .get("X-Signature")
            .ok_or((StatusCode::BAD_REQUEST, "X-Signature not found in headers"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Signature couldn't be parsed"))?;

        let tx_hash = parts
            .headers
            .get("X-Message")
            .ok_or((StatusCode::BAD_REQUEST, "X-Message not found in headers"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Message couldn't be parsed"))?;

        // Parse signature
        let signature = hex::decode(signature.trim_start_matches("0x"))
            .map_err(|_| {
                println!("Failed: Signature decode");
                (StatusCode::BAD_REQUEST, "Signature couldn't be decoded")
            })
            .and_then(|bytes| {
                PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                    println!("Failed: Signature conversion");
                    (StatusCode::BAD_REQUEST, "Signature couldn't be converted")
                })
            })?;

        // Parse message
        let tx_hash = hex::decode(tx_hash).map_err(|_| {
            println!("Failed: Message decode");
            (StatusCode::BAD_REQUEST, "Message couldn't be decoded")
        })?;

        Ok(Self {
            signature,
            tx_hash: FixedBytes::<32>::from_slice(&tx_hash),
        })
    }
}
