import type {
  CreateChannelParams,
  CreateChannelResponse,
  PaymentChannelResponse,
  RequestConfig,
  SignedRequest,
} from "./types";
import { channelFactoryABI } from "./abi/channelFactory";
import "dotenv/config";
import {
  concat,
  createPublicClient,
  createWalletClient,
  decodeEventLog,
  encodePacked,
  erc20Abi,
  http,
  keccak256,
  pad,
  parseUnits,
  toBytes,
  toHex,
  type Account,
} from "viem";
import { formatAxiosError } from "./utils";
import axios, { type InternalAxiosRequestConfig } from "axios";
import { privateKeyToAccount } from "viem/accounts";
import { baseSepolia } from "viem/chains";
import { ChannelFactoryAddress } from "./constants/address";

export class ClientInterceptor {
  private nonceMap: Map<string, number> = new Map();
  private channelStates: Map<string, PaymentChannelResponse> = new Map();

  private account!: Account;

  constructor() {
    const privateKey = process.env.WALLET_PRIVATE_KEY;

    if (!privateKey) {
      throw new Error("WALLET_PRIVATE_KEY environment variable is required");
    }

    //Intialisaing Account with viem
    this.account = privateKeyToAccount(privateKey as `0x${string}`);
    console.log("Account connected with address", this.account.address);
  }

  /**
   * Create a new payment channel
   * @param params CreateChannelParams - recipient, duration, tokenAddress, amount
   * @returns CreateChannelResponse - channelId, channelAddress, sender, recipient, duration, tokenAddress, amount, price, timestamp
   */
  async createPaymentChannel(
    params: CreateChannelParams
  ): Promise<CreateChannelResponse> {
    try {
      console.log("Creating payment channel with params:", params);

      const publicClient = createPublicClient({
        chain: baseSepolia,
        transport: http(),
      });

      const walletClient = createWalletClient({
        chain: baseSepolia,
        transport: http(),
        account: this.account,
      });

      const tokenDecimals = await publicClient.readContract({
        address: params.tokenAddress,
        abi: erc20Abi,
        functionName: "decimals",
      });

      const approveTxHash = await walletClient.writeContract({
        abi: erc20Abi,
        address: params.tokenAddress,
        functionName: "approve",
        args: [
          ChannelFactoryAddress,
          parseUnits(params.amount.toString(), tokenDecimals),
        ],
      });

      await publicClient.waitForTransactionReceipt({
        hash: approveTxHash,
      });

      // wait for 2 seconds after the approve transaction has went through
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const data = await publicClient.simulateContract({
        account: this.account,
        address: ChannelFactoryAddress,
        abi: channelFactoryABI,
        functionName: "createChannel",
        args: [
          params.recipient,
          BigInt(params.duration),
          params.tokenAddress,
          parseUnits(params.amount.toString(), tokenDecimals),
        ],
      });

      const txHash = await walletClient.writeContract(data.request);

      console.log("Transaction sent:", txHash);

      const receipt = await publicClient.waitForTransactionReceipt({
        hash: txHash,
      });

      const event = receipt.logs.find(
        (log) =>
          log.address.toLowerCase() == ChannelFactoryAddress.toLowerCase()
      );

      if (!event) {
        throw new Error("Channel creation event not found");
      }

      const eventTopics = decodeEventLog({
        abi: channelFactoryABI,
        data: event.data,
        topics: event.topics,
      });

      if (eventTopics.eventName != "channelCreated") {
        throw new Error("Channel ID not found in event logs");
      }

      console.log("Channel created:", eventTopics.args);

      return eventTopics.args;
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

  /**
   *
   * @param channelId  channel id
   * @param channelState channel state
   */
  addNewChannel(channelId: string, channelState: CreateChannelResponse) {
    this.channelStates.set(channelId, {
      address: channelState.channelAddress,
      sender: channelState.sender,
      recipient: channelState.recipient,
      balance: channelState.amount.toString(),
      nonce: "0",
      expiration: (channelState.timestamp + channelState.duration).toString(),
      channel_id: channelId,
    });
  }

  /**
   * gets the state of a payment channel
   * @param channelId
   * @returns
   */
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
   * @param paymentChannel PaymentChannelResponse
   * @param rawBody Body of the request
   * @returns SignedRequest
   */
  async signPaymentChannelRequest(
    paymentChannel: PaymentChannelResponse,
    rawBody: any
  ): Promise<SignedRequest> {
    try {
      console.log("Raw body", rawBody);

      // Convert raw body to proper format
      // Use the actual request body instead of headers
      // Empty uint8 array if no body is present
      const bodyBytes =
        rawBody == undefined
          ? new Uint8Array(0)
          : toBytes(
              typeof rawBody === "string" ? rawBody : JSON.stringify(rawBody)
            );

      console.log("Body Bytes:", bodyBytes);

      // Concatenate all parts
      const encodedMessage = keccak256(
        encodePacked(
          ["uint256", "uint256", "uint256", "bytes"],
          [
            BigInt(paymentChannel.channel_id),
            BigInt(paymentChannel.balance),
            BigInt(this.getNonce(paymentChannel.channel_id)),
            toHex(bodyBytes) as `0x${string}`,
          ]
        )
      );

      console.log("\nMessage Components:");
      console.log("Channel ID:", paymentChannel.channel_id);
      console.log("Balance:", paymentChannel.balance);
      console.log("Nonce:", this.getNonce(paymentChannel.channel_id));
      console.log("Body (hex):", toHex(bodyBytes));
      console.log("Final Message:", encodedMessage);

      // @ts-ignore
      const signature = await this.account?.signMessage({
        message: { raw: encodedMessage },
      });

      console.log("Signature:", signature);

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
   * signs a request with one time payment details
   * @param txHash Transaction Hash
   * @returns SignedRequest
   */
  async signOneTimePaymentRequest(
    txHash: `0x${string}`
  ): Promise<SignedRequest> {
    try {
      // Concatenate all parts
      const encodedMessage = keccak256(encodePacked(["bytes"], [txHash]));

      console.log("\nMessage Components:");
      console.log("Tx Hash:", txHash);
      console.log("Final Message:", encodedMessage);

      // @ts-ignore
      const signature = await this.account?.signMessage({
        message: { raw: encodedMessage },
      });

      console.log("Signature:", signature);

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
   * signs a request with stream details
   * @param sender Sender Address
   * @returns SignedRequest
   */
  async signStreamRequest(sender: `0x${string}`): Promise<SignedRequest> {
    try {
      // Concatenate all parts
      const encodedMessage = keccak256(encodePacked(["address"], [sender]));

      console.log("\nMessage Components:");
      console.log("Sender:", sender);
      console.log("Final Message:", encodedMessage);

      // @ts-ignore
      const signature = await this.account?.signMessage({
        message: { raw: encodedMessage },
      });

      console.log("Signature:", signature);

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
   * @param channelId
   */
  createPaymentChannelRequestInterceptor(channelId: string) {
    return {
      request: async (config: InternalAxiosRequestConfig) => {
        try {
          const channelState = this.channelStates.get(channelId);
          if (!channelState) {
            throw new Error(`No payment channel found for ID: ${channelId}`);
          }

          const signedRequest = await this.signPaymentChannelRequest(
            channelState,
            config.data
          );

          console.log("Adding headers to request:");
          config.headers = new axios.AxiosHeaders({
            ...config.headers,
            "x-Message": signedRequest.message,
            "x-Signature": signedRequest.signature,
            "x-Timestamp": signedRequest.timestamp,
            "x-Payment": JSON.stringify(channelState),
          });

          return config;
        } catch (err) {
          // if (axios.isAxiosError(err)) {
          //   // console.error(formatAxiosError(err));
          //   console.error("Error -kushagra2:");
          // } else {
          //   console.error("Error -kushagra2:");
          // }
          throw err;
        }
      },
    };
  }

  /**
   * creates a one time payment request interceptor for HTTP clients (axios, fetch)
   * @param txHash
   */
  createOneTimePaymentRequestInterceptor(txHash: `0x${string}`) {
    return {
      request: async (config: InternalAxiosRequestConfig) => {
        try {
          const signedRequest = await this.signOneTimePaymentRequest(txHash);

          console.log("Adding headers to request:");
          config.headers = new axios.AxiosHeaders({
            ...config.headers,
            "X-Signature": signedRequest.signature,
            "X-Transaction": txHash,
            "X-Timestamp": signedRequest.timestamp,
          });

          return config;
        } catch (err) {
          // if (axios.isAxiosError(err)) {
          //   // console.error(formatAxiosError(err));
          //   console.error("Error -kushagra2:");
          // } else {
          //   console.error("Error -kushagra2:");
          // }
          throw err;
        }
      },
    };
  }

  /**
   * creates a stream based requests interceptor for HTTP clients (axios, fetch)
   * @param sender
   */
  createStreamRequestInterceptor(sender: `0x${string}`) {
    return {
      request: async (config: InternalAxiosRequestConfig) => {
        try {
          const signedRequest = await this.signStreamRequest(sender);

          console.log("Adding headers to request:");
          config.headers = new axios.AxiosHeaders({
            ...config.headers,
            "X-Signature": signedRequest.signature,
            "X-Sender": sender,
            "X-Timestamp": signedRequest.timestamp,
          });

          return config;
        } catch (err) {
          // if (axios.isAxiosError(err)) {
          //   // console.error(formatAxiosError(err));
          //   console.error("Error -kushagra2:");
          // } else {
          //   console.error("Error -kushagra2:");
          // }
          throw err;
        }
      },
    };
  }

  /**
   * creates an response interceptor and extracts payment channel state
   */
  createPaymentChannelResponseInterceptor() {
    return {
      response: (response: any) => {
        console.log("Response Status:", response.status);
        console.log("Response Headers:", response.headers);
        console.log("Response Data:", response.data);

        // Proceed with channel state extraction
        try {
          const paymentChannelStr =
            response.headers["x-Payment"] || response.headers["x-payment"];
          if (!paymentChannelStr) {
            console.error("No payment channel found in response headers");
            return response;
          }

          const paymentChannel: PaymentChannelResponse =
            JSON.parse(paymentChannelStr);
          const channelId = paymentChannel.channel_id;

          // Update nonce
          const nextNonce = Number(paymentChannel.nonce) + 1;

          paymentChannel.nonce = nextNonce.toString();

          // Update channel state
          this.channelStates.set(channelId, paymentChannel);

          this.nonceMap.set(channelId, Number(nextNonce));

          return response;
        } catch (err) {
          // if (axios.isAxiosError(err)) {
          //   // console.error(formatAxiosError(err));
          //   console.error("Error -kushagra3:");
          // } else {
          //   // console.error("Error:", err.message);
          //   console.error("Error -kushagra3:");
          // }
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
