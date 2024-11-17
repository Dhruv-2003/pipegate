// Channel struct and implementation
// It's the local channel state for the middleware on the server side on how to store the info and just work with it

use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::transports::http::reqwest::Url;
use alloy::{
    contract::{ContractInstance, Interface},
    network::Ethereum,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::Signature,
    sol,
    transports::http::{Client, Http},
};
use tokio::sync::RwLock;

use crate::{error::AuthError, types::PaymentChannel};

// sol!(
//     #[allow(missing_docs)]
//     #[sol(rpc)]
//     ChannelFactory,
//     "src/abi/ChannelFactory.json"
// );

// sol!("../contract/src/ChannelFactory.sol",);

// sol!("../contract/src/PaymentChannel.sol");

sol! {
    contract IERC20 {
        function transferFrom(
            address sender,
            address recipient,
            uint256 amount
        ) external returns (bool);

        function transfer(
            address recipient,
            uint256 amount
        ) external returns (bool);

        function balanceOf(address account) external view returns (uint256);
    }
}

#[derive(Clone)]
pub struct ChannelState {
    pub(crate) channels: Arc<RwLock<HashMap<U256, PaymentChannel>>>, // All the channels the current server has with other user
    rate_limiter: Arc<RwLock<HashMap<Address, (u64, SystemTime)>>>,  // Rate limiter for the user
    network_rpc_url: Url, // provider: Arc<dyn Provider>, // Provider to interact with the blockchain
}

impl ChannelState {
    pub fn new(rpc_url: Url) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            network_rpc_url: rpc_url,
        }
    }

    // verification method
    pub async fn verify_signature(
        &self,
        channel_id: U256,
        signature: &Signature,
        message: &[u8],
    ) -> Result<Address, AuthError> {
        // self.network.verify_signature(signature, message).await

        // Get the address for sender from the channel state using channel id
        let channel = self.channels.read().await;
        let payment_channel = channel.get(&channel_id).ok_or(AuthError::ChannelNotFound)?;

        // Network logic to verify the signature, could be a simple ECDSA verification
        let recovered = signature.recover_address_from_msg(message);

        // Match the recovered address with the one in the channel state
        match recovered {
            Ok(address) if address == payment_channel.sender => Ok(address),
            _ => Err(AuthError::InvalidSignature),
        }
    }

    // No need in theory for validating this at the start, can do but it will add latency for a contract call on each and every API, not good
    // Can do this the first time the channel is added to the local state
    // TODO: Implement this method
    pub async fn validate_channel(
        &self,
        payment_channel: &PaymentChannel,
    ) -> Result<(), AuthError> {
        // self.network.validate_channel(channel_id, balance).await
        let provider = ProviderBuilder::new().on_http(self.network_rpc_url.clone());

        // Get the contract ABI.
        let path = std::env::current_dir()
            .unwrap()
            .join("../contract/src/PaymentChannel.sol");

        // Read the artifact which contains `abi`, `bytecode`, `deployedBytecode` and `metadata`.
        let artifact = std::fs::read(path).expect("Failed to read artifact");
        let json: serde_json::Value = serde_json::from_slice(&artifact).unwrap();

        // Get `abi` from the artifact.
        let abi_value = json.get("abi").expect("Failed to get ABI from artifact");
        let abi = serde_json::from_str(&abi_value.to_string()).unwrap();

        let payment_channel_contract: ContractInstance<Http<Client>, _, Ethereum> =
            ContractInstance::new(
                payment_channel.address,
                provider.clone(),
                Interface::new(abi),
            );

        // Fetch the balance for this payment channel from the contract implementation on the blockchain
        let balance_value = payment_channel_contract
            .function("getBalance", &[])
            .unwrap()
            .call()
            .await
            .unwrap();
        let balance = balance_value.first().unwrap().as_uint().unwrap().0;

        // If the balance is less than the balance in the local state, return an error
        if payment_channel.balance < balance {
            return Err(AuthError::InsufficientBalance);
        }

        // Fetch Expiration time for the channel from the contract
        let expiration_value = payment_channel_contract
            .function("expiration", &[])
            .unwrap()
            .call()
            .await
            .unwrap();

        let expiration = expiration_value.first().unwrap().as_uint().unwrap().0;

        if payment_channel.expiration != expiration {
            return Err(AuthError::Expired);
        }

        // Verify other data from the contract as well for the payment channel
        Ok(())
    }

    // rate limiter method
    // ✅
    pub(crate) async fn check_rate_limit(&self, sender: Address) -> Result<(), AuthError> {
        const RATE_LIMIT: u64 = 100; // 100 requests
        const WINDOW: u64 = 60; // Every 60 seconds

        let mut rate_limits = self.rate_limiter.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (count, last_reset) = rate_limits.entry(sender).or_insert((0, SystemTime::now()));

        let last_reset_secs = last_reset.duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now - last_reset_secs >= WINDOW {
            *count = 1;
            *last_reset = SystemTime::now();
            Ok(())
        } else if *count >= RATE_LIMIT {
            Err(AuthError::RateLimitExceeded)
        } else {
            *count += 1;
            Ok(())
        }
    }
}

// update method - using the insert method on the HashMap directly
