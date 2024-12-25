import {
  encodeAbiParameters,
  encodePacked,
  hashMessage,
  keccak256,
  parseAbiParameters,
  recoverMessageAddress,
  toHex,
} from "viem";
import { extractPaymentInfo } from "./utils";
import { privateKeyToAccount } from "viem/accounts";

console.log("Hello via Bun!");
async function getLog() {
  const log = {
    address: "0x9e091d62619d391b48350156c3aa1906c85c3bc6",
    topics: [
      "0xa3162614b8dec8594972fac85313f8db191ab428989960edd147302037f1f2b3",
      "0x0000000000000000000000000000000000000000000000000000000000000001",
      "0x000000000000000000000000898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33",
      "0x00000000000000000000000062c43323447899acb61c18181e34168903e033bf",
    ],
    data: "0x000000000000000000000000c51313f4d44b74b8d00b8e2b357c0e754d0539e70000000000000000000000000000000000000000000000000000000000278d00000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e00000000000000000000000000000000000000000000000000000000000f424000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000675968ce",
    blockHash:
      "0xcbb9d2728b60d206d38ea1d56df22f00c72bcfdf073e63f2677d08740ec74d8a",
    blockNumber: "0x12304f7",
    transactionHash:
      "0x85366c7efbd3557f7b7c8531d8df84f7d32426750014a5f4c98414df8cb6d877",
    transactionIndex: "0x8",
    logIndex: "0x8",
    removed: false,
  };

  const eventLog = extractPaymentInfo(log);
  console.log(eventLog);
}

const privateKey = process.env.WALLET_PRIVATE_KEY as `0x${string}`;
if (!privateKey) {
  throw new Error("Please set WALLET_PRIVATE_KEY in your environment");
}

async function extra() {
  const channelId = BigInt(1);
  const balance = BigInt(999000);
  const nonce = BigInt(1);
  const rawBody = "0x";
  console.log(rawBody);

  const encodedData = encodePacked(
    ["uint256", "uint256", "uint256", "bytes"],
    [channelId, balance, nonce, rawBody]
  );

  console.log(encodedData);
  const messageHash = keccak256(encodedData);
  console.log(messageHash);

  const account = privateKeyToAccount(privateKey);

  const hashedMessage = hashMessage({
    raw: messageHash,
  });

  console.log(hashedMessage);

  const signature = await account.signMessage({
    message: {
      raw: messageHash,
    },
  });

  console.log(signature);

  const address = await recoverMessageAddress({
    message: {
      raw: messageHash,
    },
    signature,
  });

  console.log(address);

  const txHash =
    "0xe88140d4787b1305c24961dcef2f7f73d583bb862b3cbde4b7eec854f61a0248";
  const txMessageHash = keccak256(encodePacked(["bytes"], [txHash]));

  const txSignature = await account.signMessage({
    message: {
      raw: txMessageHash,
    },
  });
  console.log(txSignature);
}

extra();
