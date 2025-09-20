export interface PaymentChannelResponse {
  address: string;
  sender: string;
  recipient: string;
  balance: string;
  nonce: string;
  expiration: string;
  channel_id: string;
}

export interface RequestConfig {
  amount: string;
  data: any;
}

export interface SignedRequest {
  message: `0x${string}`;
  signature: string;
  timestamp: string;
}

export interface CreateChannelParams {
  recipient: `0x${string}`;
  duration: number;
  tokenAddress: `0x${string}`;
  amount: number;
}

export interface CreateChannelResponse {
  channelId: bigint;
  channelAddress: `0x${string}`;
  sender: `0x${string}`;
  recipient: `0x${string}`;
  duration: bigint;
  tokenAddress: `0x${string}`;
  amount: bigint;
  price: bigint;
  timestamp: bigint;
}

export enum PaymentScheme {
  OneTime = "one-time",
  Stream = "stream",
  PaymentChannel = "channel",
}

export interface PaymentRequirements {
  scheme: PaymentScheme;
  network: string;
  amount: string;
  payTo: `0x${string}`;
  asset: `0x${string}`;
  resource: string;
  description?: string;
  maxTimeoutSeconds?: number;
  extra?: Record<string, any>;
}
