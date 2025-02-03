use std::sync::Arc;

use alloy::{
    dyn_abi::DynSolType,
    eips::BlockNumberOrTag,
    primitives::{address, Address, FixedBytes},
    providers::{Provider, ProviderBuilder, WsConnect},
    pubsub::Subscription,
    rpc::types::{Filter, Log},
};

#[derive(Debug, Clone)]
pub struct StreamListner {
    sub: Arc<Subscription<Log>>,
}

use super::{state::StreamState, StreamsConfig};

impl StreamListner {
    pub async fn new(state: StreamState, config: StreamsConfig) -> Self {
        let sub = match Self::start(state, config).await {
            Ok(sub) => sub,
            Err(e) => {
                println!("Error starting stream listener: {:?}", e);
                panic!("Error starting stream listener");
            }
        };

        Self { sub: Arc::new(sub) }
    }

    async fn start(state: StreamState, config: StreamsConfig) -> Result<Subscription<Log>, String> {
        let wss_url = "wss://base-sepolia-rpc.publicnode.com".to_string();
        let ws = WsConnect::new(wss_url);

        let provider = ProviderBuilder::new().on_ws(ws).await.map_err(|e| {
            println!("Error receiving event: {:?}", e);
            "Error receiving event".to_string()
        })?;

        let contract_address = address!("6836F23d6171D74Ef62FcF776655aBcD2bcd62Ef");
        let filter = Filter::new()
            .address(contract_address)
            .event(
                "FlowUpdated(
            address indexed token,
            address indexed sender,
            address indexed receiver,
            int96 flowRate,
            int256 totalSenderFlowRate,
            int256 totalReceiverFlowRate,
            bytes userData
        )",
            )
            .from_block(BlockNumberOrTag::Latest)
            .topic1(FixedBytes::<32>::from_slice(&config.token_address.to_vec()))
            .topic3(FixedBytes::<32>::from_slice(&config.recipient.to_vec()));

        let mut sub = provider.subscribe_logs(&filter).await.map_err(|e| {
            println!("Error receiving event: {:?}", e);
            "Error receiving event".to_string()
        })?;

        while let Ok(log) = sub.recv().await {
            println!("Received event: {:?}", log);

            if let Some(sender_topic) = log.topics().get(2) {
                let sender = Address::from_slice(&sender_topic.0[12..]);
                // check if the sender is in the stream state

                if let Some(_stream) = state.get(sender).await {
                    // check the stream flow rate and if it has changed other than the config amount, invalidate state

                    let data = &log.data().data[0..31];
                    let data_type = DynSolType::Int(96);

                    let decoded = data_type.abi_decode(data).map_err(|e| {
                        println!("Error decoding data: {:?}", e);
                        "Error decoding data".to_string()
                    })?;

                    let (flow_rate, _) = match decoded.as_int() {
                        Some(flow_rate) => flow_rate,
                        None => {
                            println!("Error parsing flow rate");
                            return Err("Error parsing flow rate".to_string());
                        }
                    };

                    if config.amount.as_i64() != flow_rate.as_i64() {
                        state.invalidate(sender).await;
                    }
                }
            }
        }

        Ok(sub)
    }
}
