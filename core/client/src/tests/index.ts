import axios from "axios";
import { PaymentChannelSDK } from "../paymentChannel";

async function testSDKInterceptors() {
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const sdk = new PaymentChannelSDK();

    const mockChannelState = {
      address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      sender: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      recipient: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      balance: "1000000000000000000",
      nonce: "0",
      expiration: Math.floor(Date.now() / 1000 + 86400).toString(),
      channel_id: "1",
    };

    // Add mock channel state to the SDK
    (sdk as any).channelStates.set(
      mockChannelState.channel_id,
      mockChannelState
    );

    const axiosInstance = axios.create({
      baseURL: "http://localhost:3000",
      timeout: 5000,
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
      },
    });

    // Attach interceptors from SDK
    axiosInstance.interceptors.request.use(
      sdk.createRequestInterceptor(mockChannelState.channel_id).request
    );
    axiosInstance.interceptors.response.use(
      sdk.createResponseInterceptor().response
    );

    console.log("\nSending request to the root route...");

    // Make a GET request to the root route
    const response = await axiosInstance.get("/", {
      validateStatus: (status) => true, // Accept any status code
    });

    console.log("\nRequest Details:");
    console.log("URL:", response.config.url);
    console.log("Method:", response.config.method);
    console.log("Headers Sent:", {
      "x-Message": response.config.headers["x-Message"],
      "x-Signature": response.config.headers["x-Signature"],
      "x-Timestamp": response.config.headers["x-Timestamp"],
      "x-Payment": response.config.headers["x-Payment"],
    });

    console.log("\nResponse Details:");
    console.log("Status:", response.status);
    console.log("Data:", response.data);

    if (response.headers["x-payment"]) {
      console.log("\nUpdated Channel State from Response:");
      const updatedChannel = JSON.parse(response.headers["x-payment"]);
      console.log("New Balance:", updatedChannel.balance);
      console.log("New Nonce:", updatedChannel.nonce);
    }

    const finalState = sdk.getChannelState(mockChannelState.channel_id);
    console.log("\nFinal SDK Channel State:", finalState);
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error("\nRequest Failed:");
      console.log("Status:", error.response?.status);
      console.log("Headers:", error.response?.headers);
      console.log("Data:", error.response?.data);
      if (error.response?.headers["x-payment"]) {
        console.log("\nChannel State in Error Response:");
        console.log(error.response.headers["x-payment"]);
      }
    } else {
      console.error("\nUnexpected Error:", error);
    }
  }
}

// Run test
console.log("=== Payment Channel SDK Interceptor Test ===");
testSDKInterceptors()
  .then(() => console.log("\nTest completed"))
  .catch(console.error);
