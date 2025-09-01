/* tslint:disable */
/* eslint-disable */
export function start(): void;
export function verify_channel_no_state(config_json: string, current_channel_json: string | undefined, message: string, signature: string, payment_channel_json: string, payment_amount: bigint, timestamp: bigint, body_bytes: Uint8Array): Promise<any>;
export function verify_onetime_payment_tx(ontime_payment_config_json: string, signature: string, tx_hash: string): Promise<any>;
export function verify_stream_tx(stream_config_json: string, signature: string, sender: string): Promise<any>;
export function close_and_withdraw_channel(rpc_url: string, private_key: string, signature: string, payment_channel_json: string, body_bytes: Uint8Array): Promise<any>;
export class PaymentChannelVerifier {
  free(): void;
  constructor(config_json: string);
  verify_request(message: string, signature: string, payment_channel_json: string, payment_amount: bigint, timestamp: bigint, body_bytes: Uint8Array): Promise<any>;
}
export class StreamVerifier {
  free(): void;
  constructor(config_json: string);
  start_listener(listener_config_json: string): void;
  verify_request(signature: string, sender: string): Promise<any>;
}
