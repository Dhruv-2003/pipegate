use std::str::FromStr;

use alloy::{
    dyn_abi::DynSolType,
    eips::BlockNumberOrTag,
    primitives::{address, Address, FixedBytes},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
};

#[derive(Debug, Clone)]
pub struct StreamListner;

use super::{state::StreamState, StreamsConfig};

impl StreamListner {
    pub async fn new(state: StreamState, config: StreamsConfig) -> Self {
        tokio::spawn(async move {
            println!("Starting event listener");
            if let Err(e) = Self::start(state, config).await {
                eprintln!("Event listener error: {:?}", e);
            }
        });

        Self
    }

    pub async fn start(state: StreamState, config: StreamsConfig) -> Result<(), String> {
        println!("Starting event listener");

        let wss_url = "wss://base-sepolia-rpc.publicnode.com".to_string();
        let ws = WsConnect::new(wss_url);

        let provider = ProviderBuilder::new().on_ws(ws).await.map_err(|e| {
            println!("Error receiving event: {:?}", e);
            "Error receiving event".to_string()
        })?;

        //     "FlowUpdated(ISuperfluid indexed token,address indexed sender,address indexed receiver,int96 flowRate,int256 totalSenderFlowRate,int256 totalReceiverFlowRate,bytes userData)"
        let event_signature = FixedBytes::<32>::from_str(
            "0x57269d2ebcccecdcc0d9d2c0a0b80ead95f344e28ec20f50f709811f209d4e0e",
        )
        .unwrap();

        let contract_address = address!("6836F23d6171D74Ef62FcF776655aBcD2bcd62Ef");

        let filter = Filter::new()
            .address(contract_address)
            .event_signature(event_signature)
            .from_block(BlockNumberOrTag::Latest)
            .topic1(FixedBytes::<32>::left_padding_from(
                &config.token_address.to_vec(),
            ))
            .topic3(FixedBytes::<32>::left_padding_from(
                &config.recipient.to_vec(),
            ));

        let mut sub = provider.subscribe_logs(&filter).await.map_err(|e| {
            println!("Error receiving event: {:?}", e);
            "Error receiving event".to_string()
        })?;

        print!("Subscribed to logs");

        while let Ok(log) = sub.recv().await {
            println!("Received event: ");

            if let Some(sender_topic) = log.topics().get(2) {
                let sender = Address::from_slice(&sender_topic.0[12..]);
                // check if the sender is in the stream state

                if let Some(_stream) = state.get(sender).await {
                    println!("Event sender in cache");
                    println!("Sender: {:?}", sender);
                    // check the stream flow rate and if it has changed other than the config amount, invalidate state

                    let data = &log.data().data[0..32];
                    let data_type = DynSolType::Int(96);

                    let decoded = match data_type.abi_decode(data) {
                        Ok(decoded) => decoded,
                        Err(e) => {
                            println!("Error decoding data: {:?}", e);
                            continue; // Skipping this event instead of breaking
                        }
                    };

                    // println!("Decoded data: {:?}", decoded);

                    let (flow_rate, _) = match decoded.as_int() {
                        Some(flow_rate) => flow_rate,
                        None => {
                            println!("Error parsing flow rate");
                            continue;
                        }
                    };

                    println!("Updated Flow rate for the sender: {:?}", flow_rate);

                    if flow_rate.as_i64() != config.amount.as_i64() {
                        println!("Invalidating stream as stream modified or cancelled");
                        state.invalidate(sender).await;
                    }
                }
            }
        }

        Ok(())
    }
}
