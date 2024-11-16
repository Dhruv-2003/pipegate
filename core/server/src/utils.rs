use alloy::primitives::U256;

pub fn create_message(channel_id: U256, balance: U256, nonce: U256, body: &[u8]) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(&channel_id.to_be_bytes::<32>());
    message.extend_from_slice(&balance.to_be_bytes::<32>());
    message.extend_from_slice(&nonce.to_be_bytes::<32>());
    message.extend_from_slice(body);
    message
}
