export interface PaymentChannelResponse {
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
  message: {
    channelId: string;
    amount: string;
    nonce: string;
    requestData: string;
    timestamp: number;
  };
  signature: string;
  timestamp: string;
}

export interface CreateChannelParams {
  recipient: string;
  duration: number;
  tokenAddress: string;
  amount: string;
}
