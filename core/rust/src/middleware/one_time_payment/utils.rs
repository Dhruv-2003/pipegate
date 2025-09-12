use alloy::{
    dyn_abi::DynSolValue,
    hex::{self},
    primitives::{keccak256, FixedBytes, PrimitiveSignature},
};
use http::HeaderMap;

use crate::{error::AuthError, middleware::one_time_payment::types::SignedPaymentTx};

pub async fn parse_tx_headers(headers: &HeaderMap) -> Result<SignedPaymentTx, AuthError> {
    let signature = headers
        .get("X-Signature")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders)?;

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

    let tx_hash = headers
        .get("X-Transaction")
        .ok_or(AuthError::MissingHeaders)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeaders)?;

    let tx_hash = hex::decode(tx_hash).map_err(|_| {
        println!("Failed: Message decode");
        AuthError::InvalidTransaction("Tx hash decode failed".to_string())
    })?;

    let signed_tx = SignedPaymentTx {
        signature,
        tx_hash: FixedBytes::<32>::from_slice(&tx_hash),
    };

    Ok(signed_tx)
}

pub fn create_tx_message(tx_hash: FixedBytes<32>) -> Vec<u8> {
    let message = DynSolValue::Tuple(vec![DynSolValue::Bytes(tx_hash.to_vec())]);

    let encoded_message = message.abi_encode_packed();

    let hashed_message = keccak256(&encoded_message);

    hashed_message.to_vec()
}
