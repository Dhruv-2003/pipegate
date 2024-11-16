import { keccak256, randomBytes, hexlify, toBeArray } from "ethers";
import { AbiCoder } from "ethers";
import type { RequestConfig, SignedRequest } from "./types";

export class PaymentChannelSDK {
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
        nonce: hexlify(randomBytes(32)),
        requestData: hexlify(bodyBytes),
        timestamp: Date.now(),
      };

      const abiCoder = AbiCoder.defaultAbiCoder();
      const messageHash = keccak256(
        abiCoder.encode(
          ["string", "string", "string", "bytes", "uint256"],
          [
            message.channelId,
            message.amount,
            message.nonce,
            bodyBytes,
            message.timestamp,
          ]
        )
      );

      // testing signature
      const signature = "0xmocksignature";

      // const signature = await this.signer.signMessage(toBeArray(messageHash));

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

          // add required headers
          config.headers = {
            ...config.headers,
            "x-signature": signedRequest.signature,
            "x-message": JSON.stringify(signedRequest.message),
            "x-timestamp": signedRequest.timestamp,
          };

          console.log("Modified headers:", config.headers);
          return config;
        } catch (err) {
          const error = err as Error;
          throw new Error(`Failed to process request: ${error.message}`);
        }
      },
    };
  }
}
