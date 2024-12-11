import { createPublicClient, createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { baseSepolia } from "viem/chains";
import { paymentChannelABI } from "../abi/abi";

const privateKey = process.env.WALLET_PRIVATE_KEY as `0x${string}`;
if (!privateKey) {
  throw new Error("Please provide a private key");
}

// This is for API Providers, to close the channel and withdraw the remaining balance in payment channel
async function main() {
  const account = privateKeyToAccount(privateKey);

  const publicClient = createPublicClient({
    chain: baseSepolia,
    transport: http(),
  });

  const walletClient = createWalletClient({
    chain: baseSepolia,
    transport: http(),
    account,
  });

  const paymentChannel = {
    address: "0xc51313f4d44b74b8d00b8e2b357c0e754d0539e7",
    sender: "0x898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33",
    recipient: "0x62c43323447899acb61c18181e34168903e033bf",
    balance: "999000",
    nonce: "1",
    expiration: "1736504782",
    channel_id: "1",
  };

  const rawBody = "0x";
  const signature =
    "0x9dbbaab8fb419ad1fc50d2d7d0c037f6621d8fc22701b92c503d80e262081d2a11343599127d064b9ca054cd0ae29c7025394f658b47b4c5c102bfd631d7bcb91b"; // Signature of the sender

  const txHash = await walletClient.writeContract({
    abi: paymentChannelABI,
    address: paymentChannel.address as `0x${string}`,
    functionName: "close",
    args: [
      BigInt(paymentChannel.balance),
      BigInt(paymentChannel.nonce),
      rawBody,
      signature,
    ],
  });

  await publicClient.waitForTransactionReceipt({
    hash: txHash,
  });
}

main();
