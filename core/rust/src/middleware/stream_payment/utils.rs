use alloy::{
    dyn_abi::DynSolValue,
    primitives::{keccak256, Address},
};
use alloy::{
    hex::{self},
    primitives::PrimitiveSignature,
};
use http::HeaderMap;
use std::str::FromStr;

use crate::{error::AuthError, middleware::stream_payment::types::SignedStream};

pub fn create_stream_message(sender: Address) -> Vec<u8> {
    let message = DynSolValue::Tuple(vec![DynSolValue::Address(sender)]);

    let encoded_message = message.abi_encode_packed();

    let hashed_message = keccak256(&encoded_message);

    hashed_message.to_vec()
}

pub async fn parse_stream_headers(headers: &HeaderMap) -> Result<SignedStream, AuthError> {
    let signature = headers
        .get("X-Signature")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders("X-Signature header contains invalid UTF-8 characters".to_string()))?;

    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| {
            println!("Failed: Signature decode");
            AuthError::InvalidSignature
        })
        .and_then(|bytes| {
            PrimitiveSignature::try_from(bytes.as_slice()).map_err(|_| {
                println!("Failed: Signature conversion");
                AuthError::InvalidSignature
            })
        })?;

    let sender = headers
        .get("X-Sender")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders("X-Sender header contains invalid UTF-8 characters".to_string()))?;

    let sender = Address::from_str(sender).map_err(|_| {
        println!("Failed: Sender conversion");
        AuthError::InvalidSender
    })?;

    let signed_tx = SignedStream {
        signature,
        sender: sender,
    };

    Ok(signed_tx)
}
