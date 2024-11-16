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
  recipient: string;
  duration: number;
  tokenAddress: string;
  amount: string;
}
