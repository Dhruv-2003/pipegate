import { ClientInterceptor } from "./client";
import {
  type CreateChannelResponse,
  type CreateChannelParams,
} from "./types/index";
import init, {
  PaymentChannelVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
  verify_onetime_payment_tx,
} from "./wasm/pipegate.js";

export {
  ClientInterceptor,
  init,
  PaymentChannelVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
  verify_onetime_payment_tx,
};
export type { CreateChannelResponse, CreateChannelParams };
