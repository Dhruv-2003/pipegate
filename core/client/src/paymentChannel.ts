import { keccak256, randomBytes, hexlify, toBeArray, Wallet } from "ethers";
import { AbiCoder } from "ethers";
import type {
  PaymentChannelResponse,
  RequestConfig,
  SignedRequest,
} from "./types";

export class PaymentChannelSDK {
  private wallet: Wallet;
  private nonceMap: Map<string, number> = new Map();

  constructor() {
    // create a test wallet with a known private key
    this.wallet = new Wallet(
      "0x1234567890123456789012345678901234567890123456789012345678901234"
    );
  }

  private getNonce(channelId: string): string {
    const currentNonce = this.nonceMap.get(channelId) || 0;
    this.nonceMap.set(channelId, currentNonce + 1);
    return currentNonce.toString();
  }

  /**
   * signs a request with channel details
   */
  async signRequest(
    request: RequestConfig,
    channelId: string,
    rawBody: any
  ): Promise<SignedRequest> {
    try {
      const bodyBytes = new TextEncoder().encode(
        typeof rawBody === "string" ? rawBody : JSON.stringify(rawBody)
      );

      const message = {
        channelId,
        amount: request.amount,
        nonce: this.getNonce(channelId), 
        requestData: hexlify(bodyBytes),
        timestamp: Date.now(),
      };

      console.log("\nMessage to be signed:", message);

      const abiCoder = AbiCoder.defaultAbiCoder();
      const encodedMessage = abiCoder.encode(
        ["string", "string", "string", "bytes", "uint256"],
        [
          message.channelId,
          message.amount,
          message.nonce,
          bodyBytes,
          message.timestamp,
        ]
      );

      const messageHash = keccak256(encodedMessage);
      const signature = await this.wallet.signMessage(toBeArray(messageHash));

      return {
        message,
        signature,
        timestamp: message.timestamp.toString(),
      };
    } catch (err) {
      const error = err as Error;
      throw new Error(`failed to sign request: ${error.message}`);
    }
  }

  /**
   * creates an interceptor for HTTP clients (axios, fetch)
   */
  createRequestInterceptor(channelId: string) {
    return {
      request: async (config: any) => {
        try {
          const rawBody = config.data;

          const signedRequest = await this.signRequest(
            {
              amount: config.headers["x-payment-amount"] || "0",
              data: config.data || {},
            },
            channelId,
            rawBody
          );

          config.headers = {
            ...config.headers,
            "x-signature": signedRequest.signature,
            "x-message": JSON.stringify(signedRequest.message),
            "x-timestamp": signedRequest.timestamp,
          };

          console.log("Request Headers:", config.headers);
          return config;
        } catch (err) {
          const error = err as Error;
          throw new Error(`Failed to process request: ${error.message}`);
        }
      },
    };
  }

  createResponseInterceptor() {
    return {
      response: (response: any) => {
        try {
          // get payment channel details from x-payment header
          const paymentChannelStr = response.headers["x-payment"];
          if (!paymentChannelStr) {
            console.log("No payment channel data in response");
            return response;
          }

          const paymentChannel: PaymentChannelResponse =
            JSON.parse(paymentChannelStr);

          // extract channelId from the original request
          const requestMessage = JSON.parse(
            response.config.headers["x-message"]
          );
          const channelId = requestMessage.channelId;

          // update nonce map with the next nonce (current nonce + 1)
          const nextNonce = BigInt(paymentChannel.nonce) + 1n;
          this.nonceMap.set(channelId, Number(nextNonce));

          console.log("\nPayment Channel Update:");
          console.log("Channel ID:", channelId);
          console.log("Current Nonce:", paymentChannel.nonce);
          console.log("Next Nonce:", nextNonce.toString());
          console.log("Balance:", paymentChannel.balance);
          console.log("Expiration:", paymentChannel.expiration);

          return response;
        } catch (err) {
          const error = err as Error;
          throw new Error(`Failed to process response: ${error.message}`);
        }
      },
    };
  }
}
