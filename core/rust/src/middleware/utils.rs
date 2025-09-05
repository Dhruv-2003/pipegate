use alloy::{
    hex,
    primitives::{Address, FixedBytes, PrimitiveSignature},
    providers::{Provider, ProviderBuilder},
    sol,
};
use std::str::FromStr;

use crate::{
    error::AuthError,
    middleware::{
        one_time_payment::types::SignedPaymentTx,
        payment_channel::types::PaymentChannel,
        stream_payment::types::{SignedStream, SUPERFLUID_TOKEN_LIST},
        types::{ChannelPayload, OneTimePayload, StreamPayload, CHAINLIST_API},
    },
};

sol! {
   // The `rpc` attribute enables contract interaction via the provider.
   #[sol(rpc)]
   contract ERC20 {
        function decimals() public view returns (uint8);
   }
}

pub async fn get_token_decimals(rpc_url: &String, token_address: &Address) -> Result<u8, String> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());

    let erc20 = ERC20::new(*token_address, provider);

    let balance = erc20.decimals().call().await;

    match balance {
        Ok(decimals) => Ok(decimals._0),
        Err(e) => Err(format!("Error fetching token decimals: {}", e)),
    }
}

pub async fn convert_signature(signature: &String) -> Result<PrimitiveSignature, AuthError> {
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
        });

    return signature;
}

pub async fn convert_tx_hash(tx_hash: &String) -> Result<FixedBytes<32>, AuthError> {
    let tx_hash = hex::decode(tx_hash).map_err(|_| {
        println!("Failed: Message decode");
        AuthError::InvalidTransaction("Tx hash decode failed".to_string())
    });

    return tx_hash.map(|h| FixedBytes::<32>::from_slice(&h));
}

pub async fn parse_onetime_payload(payload: &OneTimePayload) -> Result<SignedPaymentTx, AuthError> {
    let signature = convert_signature(&payload.signature).await?;
    let tx_hash = convert_tx_hash(&payload.tx_hash).await?;

    Ok(SignedPaymentTx { signature, tx_hash })
}

pub async fn parse_stream_payload(payload: &StreamPayload) -> Result<SignedStream, AuthError> {
    let signature = convert_signature(&payload.signature).await?;
    let sender = Address::from_str(&payload.sender).map_err(|_| AuthError::InvalidSender)?;

    Ok(SignedStream { signature, sender })
}

pub async fn parse_channel_payload(
    payload: &ChannelPayload,
) -> Result<(PrimitiveSignature, Vec<u8>, PaymentChannel), AuthError> {
    let signature = convert_signature(&payload.signature).await?;
    let message = hex::decode(&payload.message).map_err(|_| AuthError::InvalidMessage)?;

    Ok((signature, message, payload.payment_channel.clone()))
}

pub async fn get_chain_id(rpc_url: &String) -> Result<u64, AuthError> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());

    let chain_id = provider.get_chain_id().await.map_err(|e| {
        println!("Error fetching chain ID: {}", e);
        AuthError::ContractError("Failed to fetch chain ID".to_string())
    })?;

    Ok(chain_id)
}

pub async fn get_chain_name(chain_id: &u64) -> Result<String, AuthError> {
    let response = reqwest::get(CHAINLIST_API)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

    let chains = response.as_array().ok_or(AuthError::ContractError(
        "Invalid chain list format".to_string(),
    ))?;

    for chain in chains {
        if let Some(id) = chain["chainId"].as_u64() {
            if &id == chain_id {
                if let Some(name) = chain["chainSlug"].as_str() {
                    return Ok(name.to_string());
                }
            }
        }
    }
    Err(AuthError::ContractError(
        "Chain name not found for the given chain ID".to_string(),
    ))
}

pub async fn get_super_token_from_token(
    chain_id: &u64,
    token_address: &Address,
) -> Result<(Address, u8), AuthError> {
    let url = reqwest::get(SUPERFLUID_TOKEN_LIST)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

    let tokens = url["tokens"].as_array().ok_or(AuthError::ContractError(
        "Invalid token list format".to_string(),
    ))?;

    for token in tokens {
        if let Some(extensions) =
            token["extensions"]["superTokenInfo"]["underlyingTokenAddress"].as_str()
        {
            if extensions.to_lowercase() == token_address.to_string().to_lowercase()
                && chain_id == &token["chainId"].as_u64().unwrap_or(1)
            {
                let super_token_address = token["address"].as_str().ok_or(
                    AuthError::ContractError("Invalid super token address format".to_string()),
                )?;
                let decimals = token["decimals"]
                    .as_u64()
                    .and_then(|d| u8::try_from(d).ok())
                    .ok_or(AuthError::ContractError(
                        "Invalid decimals format".to_string(),
                    ))?;
                return Ok((
                    Address::from_str(super_token_address).map_err(|_| {
                        AuthError::ContractError("Invalid super token address".to_string())
                    })?,
                    decimals,
                ));
            }
        }
    }

    Err(AuthError::ContractError(
        "Super token not found for the given token".to_string(),
    ))
}

pub fn get_current_time() -> u64 {
    if cfg!(target_arch = "wasm32") {
        use js_sys::Date;

        (Date::now() / 1000.0) as u64
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
