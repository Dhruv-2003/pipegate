use alloy::{
    dyn_abi::DynSolValue,
    primitives::{keccak256, FixedBytes, U256},
};

pub fn create_channel_message(
    channel_id: U256,
    balance: U256,
    nonce: U256,
    body: &[u8],
) -> Vec<u8> {
    let message = DynSolValue::Tuple(vec![
        DynSolValue::Uint(channel_id, 256),
        DynSolValue::Uint(balance, 256),
        DynSolValue::Uint(nonce, 256),
        DynSolValue::Bytes(body.to_vec()),
    ]);

    let encoded_message = message.abi_encode_packed();

    let hashed_message = keccak256(&encoded_message);

    hashed_message.to_vec()
}

pub fn create_tx_message(tx_hash: FixedBytes<32>) -> Vec<u8> {
    let message = DynSolValue::Tuple(vec![DynSolValue::Bytes(tx_hash.to_vec())]);

    let encoded_message = message.abi_encode_packed();

    let hashed_message = keccak256(&encoded_message);

    hashed_message.to_vec()
}
