pub mod channel;
pub mod error;
pub mod middleware;
pub mod types;
pub mod utils;
pub mod verify;

mod extractors;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// #[cfg(target_arch = "wasm32")]
// pub use wasm::*;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::{Address, FixedBytes, PrimitiveSignature, U256};
    use types::{OneTimePaymentConfig, SignedPaymentTx};
    use verify::verify_tx;

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
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
