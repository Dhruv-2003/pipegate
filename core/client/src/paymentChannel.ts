import {
  keccak256,
  hexlify,
  toBeArray,
  Wallet,
  Contract,
  BrowserProvider,
  parseEther,
  ethers,
} from "ethers";
import { AbiCoder } from "ethers";
import type {
  CreateChannelParams,
  PaymentChannelResponse,
  RequestConfig,
  SignedRequest,
} from "./types";
import type { Provider } from "ethers";
import type { Signer } from "ethers";
import { channelFactoryABI } from "./abi/channel-factory";
import "dotenv/config";
import { concat, encodeAbiParameters, pad, toBytes, toHex } from "viem";
import { formatAxiosError } from "./utils";
import axios, {
  type AxiosInstance,
  type InternalAxiosRequestConfig,
} from "axios";

interface SDKConfig {
  privateKey: string;
  provider?: Provider;
  signer?: Signer;
}

export class PaymentChannelSDK {
  private wallet: Wallet;
  private nonceMap: Map<string, number> = new Map();
  private channelStates: Map<string, PaymentChannelResponse> = new Map();
  private provider!: Provider;
  private signer!: Signer;
  private channelFactory!: Contract;

  constructor() {
    const privateKey = process.env.WALLET_PRIVATE_KEY;

    if (!privateKey) {
      throw new Error("WALLET_PRIVATE_KEY environment variable is required");
    }

    this.wallet = new Wallet(privateKey);
  }

  async initialize() {
    if (!this.signer) {
      const browserProvider = this.provider as BrowserProvider;
      this.signer = await browserProvider.getSigner();
    }

    this.channelFactory = new Contract(
      "0x16b12b0002487a8FB3B3877a71Ae9258d0889E1B",
      channelFactoryABI,
      this.signer
    );
  }

  /**
   * creates a new payment channel with specified parameters
   */
  async createPaymentChannel(params: CreateChannelParams): Promise<string> {
    try {
      console.log("Creating payment channel with params:", params);

      const tx = await this.channelFactory.createChannel(
        params.recipient,
        params.duration,
        params.tokenAddress,
        parseEther(params.amount)
      );

      console.log("Transaction sent:", tx.hash);
      const receipt = await tx.wait();

      const event = receipt.logs.find(
        (log: any) => log.eventName === "channelCreated"
      );

      if (!event) {
        throw new Error("Channel creation event not found");
      }

      const channelId = event.args.channelId.toString();
      const channelAddress = event.args.channelAddress;

      console.log("Channel created:", {
        channelId,
        channelAddress,
        sender: event.args.sender,
        recipient: event.args.recipient,
        amount: event.args.amount.toString(),
        price: event.args.price.toString(),
      });

      return channelId;
    } catch (err) {
      if (axios.isAxiosError(err)) {
        console.error(formatAxiosError(err));
      } else {
        // @ts-ignore
        console.error("Error:", err.message);
      }
      throw err;
    }
  }

  // get current channel state
  getChannelState(channelId: string): PaymentChannelResponse | undefined {
    return this.channelStates.get(channelId);
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
    paymentChannel: PaymentChannelResponse,
    rawBody: any
  ): Promise<SignedRequest> {
    try {
      // Channel ID (32 bytes)
      const channelIdPadded = pad(toHex(BigInt(paymentChannel.channel_id)), {
        size: 32,
      }) as `0x${string}`;

      // Balance (32 bytes)
      const balancePadded = pad(toHex(BigInt(paymentChannel.balance)), {
        size: 32,
      }) as `0x${string}`;

      // Nonce (32 bytes)
      const noncePadded = pad(
        toHex(BigInt(this.getNonce(paymentChannel.channel_id))),
        { size: 32 }
      ) as `0x${string}`;

      // Convert raw body to proper format
      // Use the actual request body instead of headers
      const bodyBytes = toBytes(
        typeof rawBody === "string" ? rawBody : JSON.stringify(rawBody)
      );

      // Concatenate all parts
      const encodedMessage = concat([
        channelIdPadded,
        balancePadded,
        noncePadded,
        toHex(bodyBytes) as `0x${string}`,
      ]);

      console.log("\nMessage Components:");
      console.log("Channel ID:", channelIdPadded);
      console.log("Balance:", balancePadded);
      console.log("Nonce:", noncePadded);
      console.log("Body (hex):", toHex(bodyBytes));
      console.log("Final Message:", encodedMessage);

      const signature = await this.wallet.signMessage(
        toBeArray(encodedMessage)
      );

      return {
        message: encodedMessage,
        signature,
        timestamp: Math.floor(Date.now() / 1000).toString(),
      };
    } catch (err) {
      console.error("Sign Request Error:", err);
      throw err;
    }
  }

  /**
   * creates an interceptor for HTTP clients (axios, fetch)
   */
  createRequestInterceptor(channelId: string) {
    return {
      request: async (config: InternalAxiosRequestConfig) => {
        try {
          const channelState = this.channelStates.get(channelId);
          if (!channelState) {
            throw new Error(`No payment channel found for ID: ${channelId}`);
          }

          const signedRequest = await this.signRequest(
            channelState,
            config.data
          );

          config.headers = new axios.AxiosHeaders({
            ...config.headers,
            "x-Message": signedRequest.message,
            "x-Signature": signedRequest.signature,
            "x-Timestamp": signedRequest.timestamp,
            "x-Payment": JSON.stringify(channelState),
          });

          return config;
        } catch (err) {
          if (axios.isAxiosError(err)) {
            // console.error(formatAxiosError(err));
            console.error("Error -kushagra2:");
          } else {
            console.error("Error -kushagra2:");
          }
          throw err;
        }
      },
    };
  }

  /**
   * creates an response interceptor and extracts payment channel state
   */
  createResponseInterceptor() {
    return {
      response: (response: any) => {
        console.log("Response Status:", response.status);
        console.log("Response Headers:", response.headers);
        console.log("Response Data:", response.data);

        // Proceed with channel state extraction
        const paymentChannelStr = response.headers["x-Payment"];
        if (!paymentChannelStr) {
          return response;
        }

        try {
          const paymentChannelStr = response.headers["x-Payment"];
          if (!paymentChannelStr) {
            return response;
          }

          const paymentChannel: PaymentChannelResponse =
            JSON.parse(paymentChannelStr);
          const channelId = paymentChannel.channel_id;

          // Update channel state
          this.channelStates.set(channelId, paymentChannel);

          // Update nonce
          const nextNonce = BigInt(paymentChannel.nonce) + 1n;
          this.nonceMap.set(channelId, Number(nextNonce));

          return response;
        } catch (err) {
          if (axios.isAxiosError(err)) {
            // console.error(formatAxiosError(err));
            console.error("Error -kushagra3:");
          } else {
            // console.error("Error:", err.message);
            console.error("Error -kushagra3:");
          }
          throw err;
        }
      },
    };
  }

  /**
   * helper method to extract channelId from event logs
   */
  private getChannelIdFromLogs(logs: any[]): string {
    // todo: add more events based on contract spec
    const event = logs.find((log) => log.eventName === "channelCreated");

    if (!event) {
      throw new Error("Channel creation event not found in logs");
    }

    return event.args.channelId.toString();
  }
}
