import { ClientInterceptor } from "pipegate-sdk";
import { extractPaymentChannelFromResponse } from "./utils";

// This scripts is for API consumers/developers to create a payment channel
async function main() {
  const pipegateClient = new ClientInterceptor(); // Ensure the WALLET_PRIVATE_KEY is set in the .env file

  const newChannelResponse = await pipegateClient.createPaymentChannel({
    recipient: "0x3b8d0b6c7e1b1d8c7f4a1a9e7b1d8c7f4a1a9e7b",
    tokenAddress: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
    duration: 60 * 60 * 24 * 30,
    amount: 1, // depositing 1 USDC here
  });

  const paymentChannelInfo = await extractPaymentChannelFromResponse(
    newChannelResponse
  );

  console.log("Payment channel created:", paymentChannelInfo);

  // Can optionally add it in the SDK records if using the SDK
  pipegateClient.addNewChannel(
    newChannelResponse.channelId.toString(),
    newChannelResponse
  );
}
