import axios from "axios";
import { ClientInterceptor, CreateChannelResponse } from "../dist/index";

async function testSDKInterceptors() {
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const sdk = new ClientInterceptor();

    const mockChannelState = {
      address: "0x4cF93D3b7cD9D50ecfbA2082D92534E578Fe46F6",
      sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      balance: "1000000",
      nonce: "0",
      expiration: "1734391330",
      channel_id: "1",
    };

    const mockCreateChannelResponse: CreateChannelResponse = {
      channelId: 1n,
      channelAddress: "0x4cF93D3b7cD9D50ecfbA2082D92534E578Fe46F6",
      sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      duration: BigInt(60 * 60 * 24 * 30),
      tokenAddress: "0x6b175474e89094c44da98b954eedeac495271d0f",
      amount: BigInt(1000000),
      price: BigInt(1000),
      timestamp: BigInt(1731799330),
    };

    // Add mock channel state to the SDK
    sdk.addNewChannel(mockChannelState.channel_id, mockCreateChannelResponse);

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

    // Make another GET request to the root route for checking if nonce and balance works
    const response2 = await axiosInstance.get("/", {
      validateStatus: (status) => true, // Accept any status code
    });
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
