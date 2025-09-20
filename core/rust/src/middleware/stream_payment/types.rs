use alloy::primitives::{aliases::I96, Address, PrimitiveSignature};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedStream {
    pub signature: PrimitiveSignature,
    pub sender: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamsConfig {
    pub rpc_url: String,
    pub cfa_forwarder: Address,
    pub token_address: Address,
    pub recipient: Address,
    pub amount: I96,     // flowRate for the stream, without decimals per second
    pub cache_time: u64, // in seconds
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stream {
    pub sender: Address,
    pub recipient: Address,
    pub token_address: Address,
    pub flow_rate: I96,
    pub last_verified: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamListenerConfig {
    pub wss_url: String,
    pub cfa: Address,
}

pub const CFA_V1_FORWARDER_ADDRESS: &'static str = "0xcfA132E353cB4E398080B9700609bb008eceB125";
pub const SUPERFLUID_TOKEN_LIST: &'static str = "https://raw.githubusercontent.com/superfluid-finance/tokenlist/238f8f8d84c439234b533751dd98383247b23e71/superfluid.extended.tokenlist.json";
pub const SUPERFLUID_NETWORKS_LIST: &str = "https://raw.githubusercontent.com/superfluid-finance/protocol-monorepo/dev/packages/metadata/main/networks/list.cjs";
