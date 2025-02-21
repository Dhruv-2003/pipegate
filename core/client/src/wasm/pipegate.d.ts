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
  start_listener(): void;
  verify_request(signature: string, sender: string): Promise<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly start: () => void;
  readonly __wbg_paymentchannelverifier_free: (a: number, b: number) => void;
  readonly paymentchannelverifier_new: (a: number, b: number) => [number, number, number];
  readonly paymentchannelverifier_verify_request: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: bigint, i: bigint, j: number, k: number) => any;
  readonly verify_channel_no_state: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: bigint, l: bigint, m: number, n: number) => any;
  readonly verify_onetime_payment_tx: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
  readonly verify_stream_tx: (a: number, b: number, c: number, d: number, e: number, f: number) => any;
  readonly __wbg_streamverifier_free: (a: number, b: number) => void;
  readonly streamverifier_new: (a: number, b: number) => [number, number, number];
  readonly streamverifier_start_listener: (a: number) => [number, number];
  readonly streamverifier_verify_request: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly close_and_withdraw_channel: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => any;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly closure686_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure973_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
