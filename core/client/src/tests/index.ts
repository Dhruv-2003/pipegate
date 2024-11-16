import axios from "axios";
import { PaymentChannelSDK } from "../paymentChannel";

async function testInterceptor() {
  const sdk = new PaymentChannelSDK();

  // create axios instance
  const axiosInstance = axios.create();

  // add interceptor
  axiosInstance.interceptors.request.use(
    sdk.createRequestInterceptor("test-channel-id").request
  );

  try {
    // make test request
    const response = await axiosInstance.get(
      "https://api.coindesk.com/v1/bpi/currentprice.json",
      {
        headers: {
          "x-payment-amount": "0.01",
        },
        data: {
          test: "data",
        },
      }
    );

    console.log("Modified Request Headers:", response.config.headers);
    console.log("Response:", response.data);
  } catch (error) {
    console.error("Error:", error);
  }
}

testInterceptor();
