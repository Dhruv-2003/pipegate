use std::str::FromStr;

use crate::middleware::{
    one_time_payment::{
        types::{OneTimePaymentConfig, SignedPaymentTx},
        verify::verify_tx,
    },
    payment_channel::{
        channel::{close_channel, ChannelState},
        types::{PaymentChannel, PaymentChannelConfig, SignedRequest},
        verify::{verify_and_update_channel, verify_channel},
    },
    stream_payment::{
        state::StreamState,
        types::{SignedStream, StreamListenerConfig, StreamsConfig},
        verify::verify_stream,
        Stream, StreamListner,
    },
};

use alloy::{
    hex,
    primitives::{Address, Bytes, FixedBytes, PrimitiveSignature, U256},
};

use console_error_panic_hook;

use js_sys::Date;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();

    console_log::init().expect("error initializing log");
}

#[wasm_bindgen]
pub struct PaymentChannelVerifier {
    inner: ChannelState,
    config: PaymentChannelConfig,
}

#[wasm_bindgen]
impl PaymentChannelVerifier {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: String) -> Result<PaymentChannelVerifier, JsError> {
        let config: PaymentChannelConfig = serde_json::from_str(&config_json)
            .map_err(|e| JsError::new(&format!("Invalid channel config: {}", e)))?;

        Ok(PaymentChannelVerifier {
            inner: ChannelState::new(),
            config: config,
        })
    }

    #[wasm_bindgen]
    pub fn verify_request(
        &self,
        message: String,
        signature: String,
        payment_channel_json: String,
        payment_amount: u64,
        timestamp: u64,
        body_bytes: Vec<u8>,
    ) -> js_sys::Promise {
        let state = self.inner.clone();
        let config = self.config.clone();

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
                timestamp,
            };

            let result = verify_and_update_channel(&state, &config, request)
                .await
                .map_err(|e| {
                    JsValue::from_str(&format!("Verification failed: {}", e.to_string()))
                })?;

            state
                .channels
                .write()
                .await
                .insert(result.0.channel_id, result.0.clone());

            // Rate limiting is not implemented in the wasm version
            Ok(JsValue::from_str(&serde_json::to_string(&result).unwrap()))
        })
    }
}

#[wasm_bindgen]
pub fn verify_channel_no_state(
    config_json: String,
    current_channel_json: Option<String>,
    message: String,
    signature: String,
    payment_channel_json: String,
    payment_amount: u64,
    timestamp: u64,
    body_bytes: Vec<u8>,
) -> js_sys::Promise {
    future_to_promise(async move {
        let config: PaymentChannelConfig = serde_json::from_str(&config_json)
            .map_err(|e| JsError::new(&format!("Invalid config: {}", e)))?;

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

        let current_channel: Option<PaymentChannel> = current_channel_json
            .map(|json| {
                serde_json::from_str(&json)
                    .map_err(|e| JsValue::from_str(&format!("Invalid current channel: {}", e)))
            })
            .transpose()?;

        let payment_amount = U256::from(payment_amount);

        let request = SignedRequest {
            message,
            signature,
            payment_channel,
            payment_amount,
            body_bytes,
            timestamp,
        };

        let result = verify_channel(config, request, current_channel)
            .await
            .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e.to_string())))?;

        Ok(JsValue::from_str(&serde_json::to_string(&result).unwrap()))
    })
}

#[wasm_bindgen]
pub fn verify_onetime_payment_tx(
    ontime_payment_config_json: String,
    signature: String,
    tx_hash: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        let onetime_payment_config: OneTimePaymentConfig =
            serde_json::from_str(&ontime_payment_config_json).map_err(|e| {
                JsValue::from_str(&format!("Invalid onetime payment config: {}", e))
            })?;

        let signature: PrimitiveSignature = unhexlify(&signature)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))
            .and_then(|bytes| {
                PrimitiveSignature::try_from(bytes.as_slice())
                    .map_err(|_| JsValue::from_str("Invalid signature: invalid length"))
            })?;

        let tx_hash = FixedBytes::<32>::from_str(&tx_hash)
            .map_err(|e| JsValue::from_str(&format!("Invalid transaction hash: {}", e)))?;

        let signed_payment_tx = SignedPaymentTx { signature, tx_hash };

        let result = verify_tx(signed_payment_tx, onetime_payment_config)
            .await
            .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

        Ok(JsValue::from_bool(result))
    })
}

#[wasm_bindgen]
pub fn verify_stream_tx(
    stream_config_json: String,
    signature: String,
    sender: String,
) -> js_sys::Promise {
    future_to_promise(async move {
        let stream_config: StreamsConfig = serde_json::from_str(&stream_config_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid stream config: {}", e)))?;

        let signature: PrimitiveSignature = unhexlify(&signature)
            .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))
            .and_then(|bytes| {
                PrimitiveSignature::try_from(bytes.as_slice())
                    .map_err(|_| JsValue::from_str("Invalid signature: invalid length"))
            })?;

        let sender = Address::from_str(&sender)
            .map_err(|e| JsValue::from_str(&format!("Invalid sender addres: {}", e)))?;

        let signed_stream = SignedStream { signature, sender };

        let result = verify_stream(signed_stream, stream_config)
            .await
            .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

        Ok(JsValue::from_bool(result))
    })
}

#[wasm_bindgen]
pub struct StreamVerifier {
    inner: StreamState,
    config: StreamsConfig,
}

#[wasm_bindgen]
impl StreamVerifier {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: String) -> Result<StreamVerifier, JsError> {
        let config: StreamsConfig = serde_json::from_str(&config_json)
            .map_err(|e| JsError::new(&format!("Invalid stream config: {}", e)))?;

        let state = StreamState::new();

        Ok(StreamVerifier {
            inner: state,
            config: config,
        })
    }

    #[wasm_bindgen]
    pub fn start_listener(&self, listener_config_json: String) -> Result<(), JsError> {
        let listener_config: StreamListenerConfig = serde_json::from_str(&listener_config_json)
            .map_err(|e| JsError::new(&format!("Invalid stream listener config: {}", e)))?;

        let _ = StreamListner::new(self.inner.clone(), self.config.clone(), listener_config);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn verify_request(&self, signature: String, sender: String) -> js_sys::Promise {
        let state = self.inner.clone();
        let config = self.config.clone();

        future_to_promise(async move {
            let signature: PrimitiveSignature = unhexlify(&signature)
                .map_err(|e| JsValue::from_str(&format!("Invalid signature: {}", e)))
                .and_then(|bytes| {
                    PrimitiveSignature::try_from(bytes.as_slice())
                        .map_err(|_| JsValue::from_str("Invalid signature: invalid length"))
                })?;

            let sender = Address::from_str(&sender)
                .map_err(|e| JsValue::from_str(&format!("Invalid sender addres: {}", e)))?;

            let signed_stream = SignedStream { signature, sender };

            if let Some(stream) = state.get(signed_stream.sender).await {
                if stream.last_verified > 0 {
                    let timestamp = (Date::now() as u64) / 1000;

                    if timestamp - stream.last_verified < config.cache_time {
                        println!("Stream already verified, in Cache!");
                        println!("=== end middleware check ===");

                        return Ok(JsValue::from_bool(true));
                    }
                }
            }

            let result = verify_stream(signed_stream.clone(), config.clone())
                .await
                .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

            if result {
                println!("Verified");
                println!("=== end middleware check ===");

                let timestamp = (Date::now() as u64) / 1000;

                // Add the verified stream to the record
                let stream = Stream {
                    sender: signed_stream.sender,
                    recipient: config.recipient,
                    token_address: config.token_address,
                    flow_rate: config.amount,
                    last_verified: timestamp,
                };

                state.set(signed_stream.sender, stream).await;

                // start the inner subscribe/listener service for events of this sender
            }

            // Rate limiting is not implemented in the wasm version
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
