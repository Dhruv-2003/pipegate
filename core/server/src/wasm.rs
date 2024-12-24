use crate::{
    channel::{close_channel, ChannelState},
    types::{PaymentChannel, SignedRequest},
    verify::verify_and_update_channel,
};

use alloy::{
    hex,
    primitives::{Bytes, PrimitiveSignature, U256},
};

use console_error_panic_hook;
// use console_log;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen(start)]
pub fn initialize_logging() {
    console_error_panic_hook::set_once();
    // console_log::init_with_level()
}

#[wasm_bindgen]
pub struct PaymentChannelVerifier {
    inner: ChannelState,
}

#[wasm_bindgen]
impl PaymentChannelVerifier {
    #[wasm_bindgen(constructor)]
    pub fn new(rpc_url: &str) -> Result<PaymentChannelVerifier, JsError> {
        let url = rpc_url
            .parse()
            .map_err(|e| JsError::new(&format!("Invalid URL: {}", e)))?;

        Ok(PaymentChannelVerifier {
            inner: ChannelState::new(url),
        })
    }

    #[wasm_bindgen]
    pub fn verify_request(
        &self,
        message: String,
        signature: String,
        payment_channel_json: String,
        payment_amount: u64,
        body_bytes: Vec<u8>,
    ) -> js_sys::Promise {
        let state = self.inner.clone();

        future_to_promise(async move {
            let message: Vec<u8> = unhexlify(&message)
                .map_err(|e| JsValue::from_str(&format!("Invalid request: {}", e)))?;

            let signature: PrimitiveSignature = unhexlify(&signature)
                .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))
                .and_then(|bytes| {
                    PrimitiveSignature::try_from(bytes.as_slice())
                        .map_err(|_| JsValue::from_str("Invalid signature: invalid length"))
                })?;

            let payment_channel: PaymentChannel = serde_json::from_str(&payment_channel_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid payment channel: {}", e)))?;

            let payment_amount = U256::from(payment_amount);

            let request = SignedRequest {
                message,
                signature,
                payment_channel,
                payment_amount,
                body_bytes,
            };

            let result = verify_and_update_channel(&state, request)
                .await
                .map_err(|e| {
                    JsValue::from_str(&format!("Verification failed: {}", e.to_string()))
                })?;

            Ok(JsValue::from_str(&serde_json::to_string(&result).unwrap()))
        })
    }
}

#[wasm_bindgen]
pub fn close_and_withdraw_channel(
    rpc_url: String,
    private_key: String,
    signature: String,
    payment_channel_json: String,
    body_bytes: Vec<u8>,
) -> js_sys::Promise {
    future_to_promise(async move {
        let rpc_url: alloy::transports::http::reqwest::Url = rpc_url
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Invalid URL: {}", e)))?;

        let signature: PrimitiveSignature = unhexlify(&signature)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))
            .and_then(|bytes| {
                PrimitiveSignature::try_from(bytes.as_slice())
                    .map_err(|_| JsValue::from_str("Invalid signature: invalid length"))
            })?;

        let payment_channel: PaymentChannel = serde_json::from_str(&payment_channel_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid payment channel: {}", e)))?;

        close_channel(
            rpc_url,
            private_key.as_str(),
            &payment_channel,
            &signature,
            Bytes::from(body_bytes),
        )
        .await
        .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

        Ok(JsValue::NULL)
    })
}

fn _hexlify(a: &[u8]) -> String {
    let mut output = "0x".to_owned();
    output.push_str(&hex::encode(a));

    output
}

fn unhexlify(h: &String) -> Result<Vec<u8>, hex::FromHexError> {
    let mut prefix = h.to_owned();
    let s = prefix.split_off(2);
    let result = hex::decode(&s);

    result
}
