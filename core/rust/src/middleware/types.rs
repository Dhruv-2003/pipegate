use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::middleware::{
    payment_channel::types::PaymentChannel,
    utils::{get_chain_id, get_chain_name, get_super_token_from_token, get_token_decimals},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Scheme {
    #[serde(rename = "stream")]
    SuperfluidStreams, // 'streaming' payments
    #[serde(rename = "channel")]
    PaymentChannels, // 'channel' payments
    #[serde(rename = "one-time")]
    OneTimePayments, // 'exact' payments
}

impl Scheme {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "one-time" => Some(Self::OneTimePayments),
            "stream" => Some(Self::SuperfluidStreams),
            "channel" => Some(Self::PaymentChannels),
            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Self::OneTimePayments => "one-time",
            Self::SuperfluidStreams => "stream",
            Self::PaymentChannels => "channel",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PaymentPayload {
    OneTime(OneTimePayload),
    Stream(StreamPayload),
    Channel(ChannelPayload),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePayload {
    pub signature: String,
    pub tx_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamPayload {
    pub signature: String,
    pub sender: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelPayload {
    pub signature: String,
    pub message: String,
    pub payment_channel: PaymentChannel,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub accepts: Vec<SchemeConfig>,
}

impl MiddlewareConfig {
    pub fn new(accepts: Vec<SchemeConfig>) -> Self {
        Self { accepts }
    }

    pub fn add_scheme(&mut self, req: SchemeConfig) {
        self.accepts.push(req);
    }

    pub fn get_scheme_config(&self, scheme: Scheme) -> Option<&SchemeConfig> {
        self.accepts.iter().find(|config| config.scheme == scheme)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemeConfig {
    pub scheme: Scheme,
    pub network_rpc_url: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub token_address: Address,
    pub recipient: Address,
    pub amount: String,
    pub decimals: Option<u8>, // optional, if not provided will fetch from the chain
}

impl SchemeConfig {
    pub async fn new(
        scheme: Scheme,
        network_rpc_url: String,
        token_address: Address,
        recipient: Address,
        amount: String,
    ) -> Self {
        let chain_id = get_chain_id(&network_rpc_url).await.unwrap();
        let chain_name = get_chain_name(&chain_id).await.unwrap();

        // for Superfluid stream, convert the super token address
        if scheme == Scheme::SuperfluidStreams {
            let (super_token_address, decimals) =
                get_super_token_from_token(&chain_id, &token_address)
                    .await
                    .unwrap_or((token_address, 18));
            return Self {
                scheme,
                network_rpc_url,
                chain_id,
                chain_name,
                token_address: super_token_address,
                recipient,
                amount,
                decimals: Some(decimals),
            };
        } else if scheme == Scheme::PaymentChannels {
            println!("Payment Channels currently aren't safe for production, Use with caution");
        }

        let decimals = get_token_decimals(&network_rpc_url, &token_address)
            .await
            .unwrap();

        Self {
            scheme,
            network_rpc_url,
            chain_id,
            chain_name,
            token_address,
            recipient,
            amount,
            decimals: Some(decimals),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequiredAccept {
    pub scheme: String,
    pub network: String,
    pub amount: String,
    #[serde(rename = "payTo")]
    pub pay_to: String,
    pub asset: String,
    pub resource: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxTimeoutSeconds")]
    pub max_timeout_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequiredResponse {
    #[serde(rename = "x402Version")]
    pub x402_version: u8,
    pub accepts: Vec<PaymentRequiredAccept>,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentHeader {
    #[serde(rename = "x402Version")]
    pub x402_version: u64,
    pub network: String,
    pub scheme: String,
    pub payload: PaymentPayload,
}

impl PaymentHeader {
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn get_scheme_enum(&self) -> Option<Scheme> {
        Scheme::from_string(&self.scheme)
    }

    pub fn validate_payload_for_scheme(&self) -> bool {
        match (self.scheme.as_str(), &self.payload) {
            ("one-time", PaymentPayload::OneTime(_)) => true,
            ("stream", PaymentPayload::Stream(_)) => true,
            ("channel", PaymentPayload::Channel(_)) => true,
            _ => false,
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub const CHAINLIST_API: &str = "https://chainlist.org/rpcs.json";
