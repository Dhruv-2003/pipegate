use alloy::{
    dyn_abi::DynSolType,
    hex::{self},
    primitives::Address,
    providers::{Provider, ProviderBuilder},
};

use js_sys::Date;
#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use crate::{
    error::AuthError,
    middleware::one_time_payment::{
        types::{OneTimePayment, OneTimePaymentConfig, SignedPaymentTx, ABS_WINDOW_SEC},
        utils::create_tx_message,
    },
};

// For one time payment verification
pub async fn verify_tx(
    signed_tx: SignedPaymentTx,
    config: OneTimePaymentConfig,
) -> Result<(OneTimePayment, bool), AuthError> {
    // creating the message
    let reconstructed_message = create_tx_message(signed_tx.tx_hash);
    println!("Message: 0x{}", hex::encode(&reconstructed_message));

    let signature = signed_tx.signature;
    println!("Signature: 0x{}", hex::encode(&signature.as_bytes()));

    // recovering the address from the signature
    let recovered = match signature.recover_address_from_msg(reconstructed_message) {
        Ok(address) => address,
        Err(_) => return Err(AuthError::InvalidSignature),
    };
    println!("Recovered address: {}", recovered);

    let provider = ProviderBuilder::new().on_http(config.rpc_url.parse().unwrap());

    // Fetching the info for transaction
    let tx_receipt = match provider
        .get_transaction_receipt(signed_tx.tx_hash)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
    {
        Some(tx_receipt) => tx_receipt,
        None => {
            println!("Failed: Transaction not found");
            return Err(AuthError::TransactionNotFound);
        }
    };

    // Verifying recovered address against the sender for the transaction
    if recovered != tx_receipt.from {
        println!("Failed: Recovered address mismatch");
        return Err(AuthError::InvalidSignature);
    }

    // Match the contract interacted with to be the token contract
    match tx_receipt.to {
        Some(to) => {
            if to != config.token_address {
                return Err(AuthError::InvalidTransaction(
                    "Invalid token contract address".to_string(),
                ));
            }
        }
        None => {
            println!("Failed: To address not found");
            return Err(AuthError::InvalidTransaction(
                "To address not found".to_string(),
            ));
        }
    }

    let receipt = match tx_receipt.inner.as_receipt() {
        Some(receipt) => receipt,
        None => {
            return Err(AuthError::InvalidTransaction(
                "Receipt not found".to_string(),
            ))
        }
    };

    let transfer_log = match receipt.logs.first() {
        Some(log) => log,
        None => return Err(AuthError::InvalidTransaction("Log not found".to_string())),
    };

    // Check if the log is a transfer log & verify the topics and data
    if transfer_log.address() == config.token_address {
        // Verify the log timestamp is within the allowed period
        let current_time = if cfg!(target_arch = "wasm32") {
            (Date::now() / 1000.0) as u64
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        match transfer_log.block_timestamp {
            Some(timestamp) => {
                let log_time = timestamp;
                if current_time < log_time || current_time - log_time > ABS_WINDOW_SEC {
                    return Err(AuthError::InvalidTransaction(
                        "Transaction outside valid period".to_string(),
                    ));
                }
            }
            None => {
                return Err(AuthError::InvalidTransaction(
                    "Block timestamp not found".to_string(),
                ))
            }
        }

        match transfer_log.topics().get(2) {
            Some(t) => {
                let to = Address::from_word(t.clone());
                let data = &transfer_log.data().data;
                let data_type = DynSolType::Uint(256);
                let decoded = data_type
                    .abi_decode(&data)
                    .map_err(|e| AuthError::ContractError(e.to_string()))?;

                let (amount, _) = match decoded.as_uint() {
                    Some(amount) => amount,
                    None => {
                        return Err(AuthError::InvalidTransaction(
                            "Amount couldn't be parsed from event".to_string(),
                        ))
                    }
                };

                if to != config.recipient || amount != config.amount {
                    return Err(AuthError::InvalidTransaction(
                        "Invalid recipient or amount".to_string(),
                    ));
                }
            }
            None => return Err(AuthError::InvalidTransaction("Topic not found".to_string())),
        }

        let payment = OneTimePayment {
            tx_hash: signed_tx.tx_hash,
            sender: tx_receipt.from,
            payment_timestamp: if let Some(ts) = transfer_log.block_timestamp {
                ts
            } else {
                0
            },
            first_reedemed: current_time, // Set on first access
            redemptions: 0,
        };

        return Ok((payment, true));
    } else {
        return Err(AuthError::InvalidTransaction(
            "Invalid Token used for payment".to_string(),
        ));
    }
}
