import { ethers } from "ethers";
import fs from "fs";

import { decodeEventLog, fromHex } from "viem";
import { channelFactoryAbi } from "./abi";

function extractPaymentInfo(log: any): any {
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
    amount: event.args.amount.toString(),
    nonce: 0,
    price: event.args.price.toString(),
  };
}

const log = {
  address: "0xf2cabfa8b29bfb86956d1960ff748f27836e1e14",
  topics: [
    "0xa3162614b8dec8594972fac85313f8db191ab428989960edd147302037f1f2b3",
    "0x0000000000000000000000000000000000000000000000000000000000000001",
    "0x000000000000000000000000898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33",
    "0x00000000000000000000000062c43323447899acb61c18181e34168903e033bf",
  ],
  data: "0x0000000000000000000000004cf93d3b7cd9d50ecfba2082d92534e578fe46f60000000000000000000000000000000000000000000000000000000000278d00000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e00000000000000000000000000000000000000000000000000000000000f424000000000000000000000000000000000000000000000000000038d7ea4c680000000000000000000000000000000000000000000000000000000000067392922",
  blockHash:
    "0xaa9fb74cd6e32d0c2b84ef1a7ac417ed27926f06f749b0dbc13c2751d5419376",
  blockNumber: "0x112e521",
  transactionHash:
    "0x5a61767b2ea749c0b6edc774ae8466819b2b0b7c4339c9c093a2a4d3ac8e288a",
  transactionIndex: "0x8",
  logIndex: "0xf",
  removed: false,
};

const paymentInfo = extractPaymentInfo(log);

fs.writeFileSync("payment.json", JSON.stringify(paymentInfo, null, 2));

console.log("Payment info saved to payment.json:", paymentInfo);
