import axios from "axios";
import { PaymentChannelSDK } from "../paymentChannel";

async function testInterceptor() {
  const sdk = new PaymentChannelSDK();
  const axiosInstance = axios.create();

  const channelId = "test-channel-id";

  // add request and response interceptors
  axiosInstance.interceptors.request.use(
    sdk.createRequestInterceptor(channelId).request
  );

  axiosInstance.interceptors.response.use(
    sdk.createResponseInterceptor().response
  );

  // test multiple requests to see nonce increment
  const amounts = ["0.01", "0.02", "0.03"];

  for (const amount of amounts) {
    console.log(`\nMaking request with amount: ${amount}`);

    try {
      const response = await axiosInstance.get("http://localhost:3000/", {
        headers: {
          "x-payment-amount": amount,
        },
        data: {
          test: "data",
        },
      });

      console.log("\nRequest Headers:", response.config.headers);
      const message = JSON.parse(response.config.headers["x-message"]);
      console.log("\nNonce used:", message.nonce);
    } catch (error) {
      console.error("\nError:", error);
    }
  }
}

testInterceptor();
