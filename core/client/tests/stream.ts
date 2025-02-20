import axios from "axios";
import { ClientInterceptor } from "../src/index";

async function main() {
  console.log("=== Stream SDK Intercdceptor Test ===");
  console.log("\nStarting SDK Interceptor Test...");

  try {
    const sdk = new ClientInterceptor();

    const streamSender = "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33";

    const axiosInstance = axios.create({
      baseURL: "http://localhost:8000",
      timeout: 5000,
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
      },
    });

    // Attach interceptors from SDK
    axiosInstance.interceptors.request.use(
      sdk.createStreamRequestInterceptor(streamSender).request
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
      "x-Signature": response.config.headers["x-Signature"],
      "x-Sender": response.config.headers["x-Sender"],
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

main();
