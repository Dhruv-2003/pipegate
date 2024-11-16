import axios from "axios";
import { PaymentChannelSDK } from "../paymentChannel";

async function testInterceptor() {
  const sdk = new PaymentChannelSDK();
  const axiosInstance = axios.create();

  const channelId = "kushagra1213";

  // Mock channel state with seconds-based timestamp
  const mockChannelState = {
    sender: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    recipient: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    balance: "1000000000000000000", // 1 ETH in wei
    nonce: "0",
    expiration: Math.floor(Date.now() / 1000 + 86400).toString(), // 24h from now in seconds
    channel_id: channelId,
  };

  // Set the channel state manually
  (sdk as any).channelStates.set(channelId, mockChannelState);

  // add interceptors
  axiosInstance.interceptors.request.use(
    sdk.createRequestInterceptor(channelId).request
  );

  axiosInstance.interceptors.response.use(
    sdk.createResponseInterceptor().response
  );

  try {
    console.log("\nMaking request...");
    console.log("Initial channel state:", sdk.getChannelState(channelId));

    const response = await axiosInstance.get(
      "https://0335-183-88-227-115.ngrok-free.app/",
      {
        headers: {},
        data: {
          test: "data",
        },
      }
    );

    console.log("\nRequest Headers:");
    console.log("x-Message:", response.config.headers["x-Message"]);
    console.log(
      "x-Timestamp:",
      response.config.headers["x-Timestamp"],
      "(seconds)"
    );
    console.log("x-Payment:", response.config.headers["x-Payment"]);
    console.log("x-Signature:", response.config.headers["x-Signature"]);

    try {
      const paymentChannel = JSON.parse(response.config.headers["x-Payment"]);
      console.log("\nPayment Channel:", {
        ...paymentChannel,
        expiration: `${paymentChannel.expiration} (${new Date(
          paymentChannel.expiration * 1000
        ).toISOString()})`,
      });
    } catch (e) {
      console.log("Could not parse payment channel data");
    }
  } catch (error) {
    console.error("\nError:", error);
  }

  console.log("\nFinal channel state:", sdk.getChannelState(channelId));
}

console.log("Starting test...");
testInterceptor().catch(console.error);
