pub mod channel;
pub mod error;
pub mod middleware;
pub mod types;
pub mod utils;
pub mod verify;

mod extractors;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::{aliases::I96, Address, FixedBytes, PrimitiveSignature, U256};
    use types::{
        tx::{SignedStream, StreamsConfig},
        OneTimePaymentConfig, SignedPaymentTx,
    };
    use verify::{verify_stream, verify_tx};

    use super::*;

    #[tokio::test]
    async fn test_verify_and_update() {}

    #[tokio::test]
    async fn test_verify_tx() {
        let rpc_url = "https://base-sepolia-rpc.publicnode.com";

        let onetime_payment_config = OneTimePaymentConfig {
            recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
            token_address: Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
            amount: U256::from(1000000), // 1 USDC
            period: U256::from(0),
            rpc_url: rpc_url.to_string(),
        };

        let signed_payment_tx = SignedPaymentTx {
            signature: PrimitiveSignature::from_str("0xe3ebb83954309b86cc6d27e7e70b5dbcb0447cf79f8d74fc3806a6e814138fb573d3df3c1fcae6fd8fe1dca34ba8bb2748da3b68790df8ce45108016b601c12a1b").unwrap(),
            tx_hash: FixedBytes::<32>::from_str("0xe88140d4787b1305c24961dcef2f7f73d583bb862b3cbde4b7eec854f61a0248").unwrap(),
        };

        let result = verify_tx(signed_payment_tx, onetime_payment_config).await;
        println!("Result: {:?}", result);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_stream() {
        let rpc_url = "https://base-sepolia-rpc.publicnode.com";

        let stream_payment_config = StreamsConfig {
            recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
            token_address: Address::from_str("0x1650581f573ead727b92073b5ef8b4f5b94d1648").unwrap(),
            amount: "761035007610".parse::<I96>().unwrap(), // 2 USDC per month
            cfa_forwarder: Address::from_str("0xcfA132E353cB4E398080B9700609bb008eceB125").unwrap(),
            rpc_url: rpc_url.to_string(),
        };

        let signed_stream = SignedStream {
            signature: PrimitiveSignature::from_str("0x9dce84f7bd5fea33c7d91042f8fd5ee539d8c4ed9dcfcd49884ae1cb99842a8c4fa243b75eb3fd2d611e953e40202a5e94a3c513268f75a002f89c5a375527231b").unwrap(),
            sender: Address::from_str("0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33").unwrap(),
        };

        let result = verify_stream(signed_stream, stream_payment_config).await;
        println!("Result: {:?}", result);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
