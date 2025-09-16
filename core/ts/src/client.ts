import {
  PaymentScheme,
  type CreateChannelParams,
  type CreateChannelResponse,
  type PaymentChannelResponse,
  type PaymentRequirements,
  type RequestConfig,
  type SignedRequest,
} from "./types/index.js";
import { channelFactoryABI } from "./abi/channelFactory.js";
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
import { formatAxiosError } from "./utils/index.js";
import axios, { AxiosInstance, type InternalAxiosRequestConfig } from "axios";
import { privateKeyToAccount } from "viem/accounts";
import { baseSepolia } from "viem/chains";
import { ChannelFactoryAddress } from "./constants/address.js";

export class ClientInterceptor {
  private channelStates: Map<string, PaymentChannelResponse> = new Map();

  private account!: Account;

  constructor(pkey?: `0x${string}`) {
    let privateKey = process.env.WALLET_PRIVATE_KEY || pkey;

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

      if (eventTopics.eventName != "ChannelCreated") {
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

  updateChannel(channelId: string, channelState: PaymentChannelResponse) {
    this.channelStates.set(channelId, channelState);
  }

  /**
   * gets the state of a payment channel
   * @param channelId
   * @returns
   */
  getChannelState(channelId: string): PaymentChannelResponse | undefined {
    return this.channelStates.get(channelId);
  }

  getNonce(channelId: string): string {
    const channelState = this.channelStates.get(channelId);
    return channelState?.nonce || "0";
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
            BigInt(paymentChannel.nonce),
            toHex(bodyBytes) as `0x${string}`,
          ]
        )
      );

      console.log("\nMessage Components:");
      console.log("Channel ID:", paymentChannel.channel_id);
      console.log("Balance:", paymentChannel.balance);
      console.log("Nonce:", paymentChannel.nonce);
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

          config.headers.set({
            "x-Message": signedRequest.message,
            "x-Signature": signedRequest.signature,
            "x-Timestamp": signedRequest.timestamp,
            "x-Payment": JSON.stringify(channelState),
          });

          return config;
        } catch (err) {
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
          config.headers.set({
            "X-Signature": signedRequest.signature,
            "X-Transaction": txHash,
            "X-Timestamp": signedRequest.timestamp,
          });

          return config;
        } catch (err) {
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

          config.headers.set({
            "X-Signature": signedRequest.signature,
            "X-Sender": sender,
            "X-Timestamp": signedRequest.timestamp,
          });

          return config;
        } catch (err) {
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

          return response;
        } catch (err) {
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

/**
 *
 * Enables the payment of APIs using the x402 payment protocol with different supported schemes.
 *
 * @note This function logic is largely inspired from "x402-axios" package written by the coinbase team
 * @param axiosClient - The Axios instance to add the interceptor to
 * @param privateKey - A private key that can sign and create payment headers
 * @param config - Configuration for the payment interceptor
 * @returns
 */
export function withPaymentInterceptor(
  axiosClient: AxiosInstance,
  privateKey: `0x${string}`,
  config: {
    channel?: CreateChannelResponse;
    oneTimePaymentTxHash?: `0x${string}`;
    streamSender?: `0x${string}`;
  }
) {
  if (!privateKey) {
    throw new Error("Private key is required for payment interceptor");
  }

  const client = new ClientInterceptor(privateKey);

  if (config.channel) {
    client.addNewChannel(config.channel.channelId.toString(), config.channel);
  }

  axiosClient.interceptors.response.use(
    (response) => {
      const paymentChannelStr =
        response.headers["X-Payment"] || response.headers["x-payment"];
      if (!paymentChannelStr) {
        return response;
      }

      const paymentChannel: PaymentChannelResponse =
        JSON.parse(paymentChannelStr);
      const channelId = paymentChannel.channel_id;
      const nextNonce = Number(paymentChannel.nonce) + 1;

      paymentChannel.nonce = nextNonce.toString();
      client.updateChannel(channelId, paymentChannel);

      return response;
    },
    async (error) => {
      if (!error.response || error.response.status !== 402) {
        return Promise.reject(error);
      }
      try {
        //1. Check if we have already retried this request
        const originalConfig = error.config;

        if (!originalConfig || !originalConfig.headers) {
          return Promise.reject(
            new Error("Missing axios request configuration")
          );
        }

        if ((originalConfig as { __is402Retry?: boolean }).__is402Retry) {
          return Promise.reject(error);
        }

        //2. Parse the respone to extract payment requirements
        const { x402Version, accepts } = error.response.data as {
          x402Version: number;
          accepts: PaymentRequirements[];
        };

        //3. Choose the payment requirement based on user provided config (scheme-aware selection)
        //   Priority: explicit user-provided capability order of discovery
        //   We build a desired schemes list from provided config and pick the first match in that order.
        const desiredSchemes: PaymentScheme[] = [];
        if (config.oneTimePaymentTxHash)
          desiredSchemes.push(PaymentScheme.OneTime);
        if (config.channel) desiredSchemes.push(PaymentScheme.PaymentChannel);
        if (config.streamSender) desiredSchemes.push(PaymentScheme.Stream);

        let paymentRequirement: PaymentRequirements | undefined;

        if (desiredSchemes.length > 0) {
          for (const scheme of desiredSchemes) {
            paymentRequirement = accepts.find((r) => r.scheme === scheme);
            if (paymentRequirement) break;
          }
        }

        // Fallback: if nothing matched user config, take first (maintains backward behaviour)
        if (!paymentRequirement) paymentRequirement = accepts[0];

        if (!paymentRequirement) {
          return Promise.reject(
            new Error(
              "No acceptable payment requirements found from server response"
            )
          );
        }

        //4. Create payment header
        let paymentHeader = {
          x402Version: x402Version,
          network: paymentRequirement.network,
          scheme: paymentRequirement.scheme,
          payload: {},
        };

        if (paymentRequirement.scheme === PaymentScheme.OneTime) {
          if (config.oneTimePaymentTxHash === undefined) {
            return Promise.reject(
              new Error("One time payment transaction hash is required")
            );
          }

          const signedRequest = await client.signOneTimePaymentRequest(
            config.oneTimePaymentTxHash
          );

          paymentHeader.payload = {
            signature: signedRequest.signature,
            tx_hash: config.oneTimePaymentTxHash,
          };
        } else if (paymentRequirement.scheme === PaymentScheme.Stream) {
          if (config.streamSender === undefined) {
            return Promise.reject(
              new Error("Stream sender address is required")
            );
          }

          const signedRequest = await client.signStreamRequest(
            config.streamSender
          );
          paymentHeader.payload = {
            signature: signedRequest.signature,
            sender: config.streamSender,
          };
        } else if (paymentRequirement.scheme === PaymentScheme.PaymentChannel) {
          if (config.channel === undefined) {
            return Promise.reject(
              new Error("Payment channel ID is required for channel payments")
            );
          }

          const channelState = client.getChannelState(
            config.channel.channelId.toString()
          );
          if (!channelState) {
            return Promise.reject(
              new Error(
                `No payment channel found for ID: ${config.channel.channelId.toString()}`
              )
            );
          }

          const signedRequest = await client.signPaymentChannelRequest(
            channelState,
            undefined
          );

          paymentHeader.payload = {
            signature: signedRequest.signature,
            message: signedRequest.message,
            paymentChannel: {
              address: channelState.address,
              sender: channelState.sender,
              recipient: channelState.recipient,
              balance: channelState.balance.toString(),
              nonce: channelState.nonce.toString(),
              expiration: channelState.expiration.toString(),
              channel_id: channelState.channel_id.toString(),
            },
            timestamp: Number(signedRequest.timestamp),
          };
        }

        //4. Retry the original request with payment header
        (originalConfig as { __is402Retry?: boolean }).__is402Retry = true;

        originalConfig.headers["X-Payment"] = JSON.stringify(paymentHeader);
        originalConfig.headers["Access-Control-Expose-Headers"] =
          "X-PAYMENT-RESPONSE";

        //5. Expose the X-PAYMENT-RESPONSE header in the final response
        const secondResponse = await axiosClient.request(originalConfig);
        return secondResponse;
      } catch (err) {
        return Promise.reject(err);
      }
    }
  );

  return axiosClient;
}
