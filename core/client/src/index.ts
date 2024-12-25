import { ClientInterceptor } from "./client";
import {
  type CreateChannelResponse,
  type CreateChannelParams,
} from "./types/index";
import init, {
  PaymentChannelVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
} from "./wasm/pipegate.js";

export {
  ClientInterceptor,
  init,
  PaymentChannelVerifier,
  verify_channel_no_state,
  close_and_withdraw_channel,
};
export type { CreateChannelResponse, CreateChannelParams };
