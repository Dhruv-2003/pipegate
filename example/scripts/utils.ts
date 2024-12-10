import { ethers } from "ethers";
import fs from "fs";

import { decodeEventLog, fromHex } from "viem";
import { channelFactoryAbi } from "../abi/abi";
import type { CreateChannelResponse } from "pipegate-sdk";
import type { PaymentChannelResponse } from "pipegate-sdk/dist/types";

export function extractPaymentInfo(log: any): any {
  // remove 24 bytes of padding from data

  const event = decodeEventLog({
    abi: channelFactoryAbi,
    data: log.data,
    topics: log.topics,
  });

  if (event.eventName !== "channelCreated") {
    throw new Error("Invalid event name");
    return;
  }

  // Construct payment object
  return {
    channel_id: event.args.channelId.toString(),
    address: event.args.channelAddress,
    sender: event.args.sender,
    recipient: event.args.recipient,
    duration: event.args.duration.toString(),
    tokenAddress: event.args.tokenAddress,
    balance: event.args.amount.toString(),
    nonce: 0,
    price: event.args.price.toString(),
    expiration: (event.args.timestamp + event.args.duration).toString(),
  };
}

export function extractPaymentChannelFromResponse(
  res: CreateChannelResponse
): PaymentChannelResponse {
  return {
    address: res.channelAddress,
    sender: res.sender,
    recipient: res.recipient,
    balance: res.amount.toString(),
    nonce: "0",
    expiration: (res.timestamp + res.duration).toString(),
    channel_id: res.channelId.toString(),
  };
}
