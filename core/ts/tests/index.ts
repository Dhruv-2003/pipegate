import axios from "axios";
import {
  ClientInterceptor,
  CreateChannelResponse,
  withPaymentInterceptor,
} from "../src/index";

const PRIVATE_KEY = process.env.WALLET_PRIVATE_KEY || "0x";

async function tesțPaymentMiddlewareOneTime() {
  console.log("=== Payment middleware Test ===");
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const sdk = new ClientInterceptor();

    const streamSender = "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33";

    const txHash =
      "0xbcbb19b148faf983d2486f172503d7578f8306d748775d9a6cf6b876a4b56849";

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

// Run test
async function main() {
  tesțPaymentMiddlewareOneTime()
    .then(() => console.log("\nTest completed"))
    .catch(console.error);
}

main();
