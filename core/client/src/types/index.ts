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
