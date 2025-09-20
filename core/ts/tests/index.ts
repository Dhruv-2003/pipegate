import axios from "axios";
import {
  ClientInterceptor,
  CreateChannelResponse,
  withPaymentInterceptor,
} from "../src/index";

// @ts-ignore
const PRIVATE_KEY = process.env.WALLET_PRIVATE_KEY || "0x";

async function testPaymentMiddlewareOneTime() {
  console.log("=== Payment middleware Test ===");
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const txHash =
      "0x1a06852ea01b05d1d311dc51c3b34b03258a97b8c1ef790435a0b6feca8dc2c2";

    const axiosInstance = withPaymentInterceptor(
      axios.create({
        baseURL: "http://localhost:8000",
        timeout: 5000,
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
      }),
      PRIVATE_KEY,
      {
        oneTimePaymentTxHash: txHash,
      }
    );

    console.log("\nSending request to the /test route...");

    // Make a GET request to the root route
    const response = await axiosInstance.get("/test");

    console.log("\nRequest Details:");
    console.log("URL:", response.config.url);
    console.log("Method:", response.config.method);
    console.log("Headers Sent:", {
      "x-Payment": response.config.headers["x-Payment"],
    });

    console.log("\nResponse Details:");
    console.log("Status:", response.status);
    console.log("Data:", response.data);
    console.log("Data:", response.statusText);
  } catch (error) {
    console.log(error);
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

async function testPaymentMiddlewareStream() {
  console.log("=== Payment middleware Test ===");
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const streamSender = "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33";

    const axiosInstance = withPaymentInterceptor(
      axios.create({
        baseURL: "http://localhost:8000",
        timeout: 5000,
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
      }),
      PRIVATE_KEY,
      {
        streamSender: streamSender,
      }
    );

    console.log("\nSending request to the /test route...");

    // Make a GET request to the root route
    const response = await axiosInstance.get("/test");

    console.log("\nRequest Details:");
    console.log("URL:", response.config.url);
    console.log("Method:", response.config.method);
    console.log("Headers Sent:", {
      "x-Payment": response.config.headers["x-Payment"],
    });

    console.log("\nResponse Details:");
    console.log("Status:", response.status);
    console.log("Data:", response.data);
    console.log("Data:", response.statusText);
  } catch (error) {
    console.log(error);
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

async function testPaymentMiddlewareChannel() {
  console.log("=== Payment middleware Test ===");
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const mockCreateChannelResponse: CreateChannelResponse = {
      channelId: 1n,
      sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      channelAddress: "0x97cefbdf03f430fc7009fb7ebe2b87c45bf551d1",
      duration: 2592000n,
      tokenAddress: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      amount: 1000000n,
      price: 10000n,
      timestamp: 1757961238n,
    };

    const axiosInstance = withPaymentInterceptor(
      axios.create({
        baseURL: "http://localhost:8000",
        timeout: 5000,
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
      }),
      PRIVATE_KEY,
      {
        channel: mockCreateChannelResponse,
      }
    );

    console.log("\nSending request to the /test route...");

    // Make a GET request to the root route
    const response = await axiosInstance.get("/test");

    console.log("\nRequest Details:");
    console.log("URL:", response.config.url);
    console.log("Method:", response.config.method);
    console.log("Headers Sent:", {
      "x-Payment": response.config.headers["x-Payment"],
    });

    console.log("\nResponse Details:");
    console.log("Status:", response.status);
    console.log("Data:", response.data);
    console.log("Data:", response.statusText);

    const response2 = await axiosInstance.get("/test");
    console.log("\nSecond Request Details:");
    console.log("Status:", response2.status);
    console.log("Data:", response2.data);
  } catch (error) {
    console.log(error);
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
async function main() {
  // start time
  let start = Date.now();

  await testPaymentMiddlewareOneTime()
    .then(() => console.log("\nTest completed"))
    .catch(console.error);

  await testPaymentMiddlewareStream()
    .then(() => console.log("\nTest completed"))
    .catch(console.error);

  await testPaymentMiddlewareChannel()
    .then(() => console.log("\nTest completed"))
    .catch(console.error);

  let end = Date.now();
  console.log("Time taken: ", end - start, "ms");
}

main();
