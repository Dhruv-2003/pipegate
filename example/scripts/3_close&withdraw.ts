import { createPublicClient, createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { baseSepolia } from "viem/chains";
import { paymentChannelABI } from "../abi/abi";

const privateKey = process.env.PRIVATE_KEY as `0x${string}`;
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
    address: "0x4cf93d3b7cd9d50ecfba2082d92534e578fe46f6",
    sender: "0x898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33",
    recipient: "0x62c43323447899acb61c18181e34168903e033bf",
    balance: "999000",
    nonce: "1",
    expiration: "1734391330",
    channel_id: "1",
  };

  const initialBalance = await publicClient.readContract({
    abi: paymentChannelABI,
    address: paymentChannel.address as `0x${string}`,
    functionName: "getBalance",
  });

  const totalAmountToWithdraw = initialBalance - BigInt(paymentChannel.balance);

  const rawBody = "0x";
  const signature = "0x..."; // Signature of the sender

  const txHash = await walletClient.writeContract({
    abi: paymentChannelABI,
    address: paymentChannel.address as `0x${string}`,
    functionName: "close",
    args: [
      totalAmountToWithdraw,
      BigInt(paymentChannel.nonce),
      rawBody,
      signature,
    ],
  });

  await publicClient.waitForTransactionReceipt({
    hash: txHash,
  });
}
