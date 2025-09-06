import { ClientInterceptor, withPaymentInterceptor } from "./client.js";
import {
  type CreateChannelResponse,
  type CreateChannelParams,
} from "./types/index";
import {
  PaymentChannelVerifier,
  StreamVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
  verify_onetime_payment_tx,
  verify_stream_tx,
} from "./wasm/pipegate.js";
import init from "./wasm/pipegate.js";

export {
  ClientInterceptor,
  withPaymentInterceptor,
  init as initWasm,
  PaymentChannelVerifier,
  StreamVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
  verify_onetime_payment_tx,
  verify_stream_tx,
};
export type { CreateChannelResponse, CreateChannelParams };
