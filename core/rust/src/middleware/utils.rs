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
        stream_payment::types::{SignedStream, SUPERFLUID_NETWORKS_LIST, SUPERFLUID_TOKEN_LIST},
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

pub async fn get_chain_wss_url(chain_id: &u64) -> Result<String, AuthError> {
    match chain_id {
        1 => return Ok("wss://ethereum-rpc.publicnode.com".to_string()),
        137 => return Ok("wss://polygon-bor-rpc.publicnode.com".to_string()),
        42161 => return Ok("wss://arbitrum-one.publicnode.com".to_string()),
        10 => return Ok("wss://optimism-rpc.publicnode.com".to_string()),
        8453 => return Ok("wss://base-rpc.publicnode.com".to_string()),
        _ => {}
    }

    // Fallback: query Chainlist style aggregate (same endpoint already used by get_chain_name)
    let response = reqwest::get(CHAINLIST_API)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

    let chains = response.as_array().ok_or(AuthError::ContractError(
        "Invalid chain list format".to_string(),
    ))?;

    // Find chain object
    let chain_obj = chains
        .iter()
        .find(|c| c.get("chainId").and_then(|v| v.as_u64()) == Some(*chain_id))
        .ok_or_else(|| AuthError::ContractError(format!("Chain id {} not found", chain_id)))?;

    // Extract rpc array (can contain strings or objects with { url: ... })
    let mut candidates: Vec<String> = Vec::new();
    if let Some(rpc_entries) = chain_obj.get("rpc") {
        if let Some(arr) = rpc_entries.as_array() {
            for entry in arr {
                if let Some(url) = entry.as_str() {
                    if url.starts_with("wss://") && !url.contains("{") {
                        candidates.push(url.to_string());
                    }
                } else if let Some(obj_url) = entry.get("url").and_then(|v| v.as_str()) {
                    if obj_url.starts_with("wss://") && !obj_url.contains("{") {
                        candidates.push(obj_url.to_string());
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        return Err(AuthError::ContractError(format!(
            "No wss endpoints found for chain id {}",
            chain_id
        )));
    }

    // Probe each candidate by attempting an HTTPS GET on the same host (cheap liveness heuristic)
    // Convert wss://host/path -> https://host/path. If the HTTPS endpoint responds (status < 500), accept.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

    for wss in &candidates {
        // Derive probe URL
        let probe = if let Some(rest) = wss.strip_prefix("wss://") {
            format!("https://{}", rest)
        } else {
            continue;
        };

        // Try a lightweight GET (head sometimes blocked); ignore body
        if let Ok(resp) = client.get(&probe).send().await {
            if resp.status().as_u16() < 500 {
                return Ok(wss.clone());
            }
        }
    }

    // Fallback to first candidate if probing failed
    Ok(candidates.remove(0))
}

pub async fn get_chain_name(chain_id: &u64) -> Result<String, AuthError> {
    // Lookup match records first before falling back to chainlist
    match chain_id {
        // Ethereum mainnets
        1 => return Ok("ethereum".to_string()),
        137 => return Ok("polygon".to_string()),
        42161 => return Ok("arbitrum".to_string()),
        10 => return Ok("optimism".to_string()),
        56 => return Ok("bsc".to_string()),
        8453 => return Ok("base".to_string()),
        43114 => return Ok("avalanche".to_string()),
        42220 => return Ok("celo".to_string()),
        324 => return Ok("zksync-era".to_string()),

        // Testnets
        11155111 => return Ok("sepolia".to_string()),
        421614 => return Ok("arbitrum-sepolia".to_string()), // replaces Goerli
        84532 => return Ok("base-sepolia".to_string()),
        11155420 => return Ok("optimism-sepolia".to_string()),
        43113 => return Ok("avalanche-fuji".to_string()),
        44787 => return Ok("celo-alfajores".to_string()),

        _ => {}
    }

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
                if let Some(name) = chain["shortName"].as_str() {
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

/// Fetch the Constant Flow Agreement (CFAv1) contract address for a given `chain_id`.
/// Source: Superfluid protocol metadata networks list (CommonJS array export)
pub async fn get_cfa_from_chain_id(chain_id: &u64) -> Result<Address, AuthError> {
    let raw = reqwest::get(SUPERFLUID_NETWORKS_LIST)
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?
        .text()
        .await
        .map_err(|e| AuthError::ContractError(e.to_string()))?;

    // The file starts with optional comments and `module.exports =` then an array literal.
    // We extract the JSON array substring heuristically.
    let start = raw.find('[').ok_or_else(|| {
        AuthError::ContractError("Invalid networks list format (no '[')".to_string())
    })?;
    let end = raw.rfind(']').ok_or_else(|| {
        AuthError::ContractError("Invalid networks list format (no closing ']')".to_string())
    })?;
    let array_slice = &raw[start..=end];

    let networks: serde_json::Value = serde_json::from_str(array_slice)
        .map_err(|e| AuthError::ContractError(format!("Failed to parse networks list: {}", e)))?;

    let networks_arr = networks.as_array().ok_or_else(|| {
        AuthError::ContractError("Networks list root is not an array".to_string())
    })?;

    for net in networks_arr {
        let Some(id) = net.get("chainId").and_then(|v| v.as_u64()) else {
            continue;
        };
        if &id != chain_id {
            continue;
        }

        // Prefer contractsV1.cfaV1; if absent, fall back to possible key variations.
        if let Some(cfa_addr_str) = net
            .get("contractsV1")
            .and_then(|c| c.get("cfaV1"))
            .and_then(|v| v.as_str())
        {
            let addr = Address::from_str(cfa_addr_str)
                .map_err(|_| AuthError::ContractError("Invalid CFA address format".to_string()))?;
            return Ok(addr);
        }
        // If not found provide a clearer error.
        return Err(AuthError::ContractError(format!(
            "CFA address not found in contractsV1 for chain id {}",
            id
        )));
    }

    Err(AuthError::ContractError(format!(
        "Chain id {} not present in Superfluid networks list",
        chain_id
    )))
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
