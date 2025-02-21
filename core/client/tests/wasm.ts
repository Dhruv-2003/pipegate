import {
  initWasm,
  PaymentChannelVerifier,
  StreamVerifier,
  verify_onetime_payment_tx,
} from "../src";

const dummyPaymentChannel = {
  address: "0xC72DfAC1a7B3Bc178F10Dc3bf36c7F64cf41B7DE",
  sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
  recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
  balance: "1000000",
  nonce: "0",
  expiration: "1736528914",
  channel_id: "2",
};

const dummyPaymentChannelCall = {
  message: "0xf1e267d9580d52e784d9681653dd37ad9674780da936fcd265e46e10d333b42e",
  signature:
    "0x4de2d31f6bd2aab04725c3e454c2d4b6fc5485c920e66465d55894c25d7c9688005571179ee04c1570c5740712ca6e7a428da04c91270cad1aa9671dc17ba28b1c",
  paymentChannelJSON: JSON.stringify(dummyPaymentChannel),
  paymentAmount: BigInt(1000),
  timestamp: BigInt(1735387288),
  bodyBytes: new Uint8Array(0),
};

async function verifyPaymentChannelCall() {
  try {
    await initWasm();

    let config_json = {
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      token_address: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      amount: "1000",
      rpc_url: "https://base-sepolia-rpc.publicnode.com",
    };

    const verifier = new PaymentChannelVerifier(JSON.stringify(config_json));

    const updatedChannel = await verifier.verify_request(
      dummyPaymentChannelCall.message,
      dummyPaymentChannelCall.signature,
      dummyPaymentChannelCall.paymentChannelJSON,
      dummyPaymentChannelCall.paymentAmount,
      dummyPaymentChannelCall.timestamp,
      dummyPaymentChannelCall.bodyBytes
    );

    console.log(updatedChannel);
  } catch (e) {
    console.error(e);
  }
}

const dummyStreamCall = {
  signature:
    "0x9dce84f7bd5fea33c7d91042f8fd5ee539d8c4ed9dcfcd49884ae1cb99842a8c4fa243b75eb3fd2d611e953e40202a5e94a3c513268f75a002f89c5a375527231b",
  sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
};

async function verifyStreamsCall() {
  try {
    await initWasm();

    let config_json = {
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      token_address: "0x1650581f573ead727b92073b5ef8b4f5b94d1648",
      cfa_forwarder: "0xcfA132E353cB4E398080B9700609bb008eceB125",
      rpc_url: "https://base-sepolia-rpc.publicnode.com",
      amount: "761035007610",
      cache_time: 900,
    };

    const verifier = new StreamVerifier(JSON.stringify(config_json));

    let listener_config_json = {
      wss_url: "wss://base-sepolia-rpc.publicnode.com",
      cfa: "0x6836F23d6171D74Ef62FcF776655aBcD2bcd62Ef",
    };

    verifier.start_listener(JSON.stringify(listener_config_json));

    const updatedChannel = await verifier.verify_request(
      dummyStreamCall.signature,
      dummyStreamCall.sender
    );

    console.log(updatedChannel);
  } catch (e) {
    console.error(e);
  }
}

const dummyOneTimeCall = {
  signature:
    "0xe3ebb83954309b86cc6d27e7e70b5dbcb0447cf79f8d74fc3806a6e814138fb573d3df3c1fcae6fd8fe1dca34ba8bb2748da3b68790df8ce45108016b601c12a1b",
  tx_hash: "0xe88140d4787b1305c24961dcef2f7f73d583bb862b3cbde4b7eec854f61a0248",
};

async function verifyOneTimePayment() {
  try {
    await initWasm();
    let config_json = {
      recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
      token_address: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      rpc_url: "https://base-sepolia-rpc.publicnode.com",
      amount: "1000000",
      period: 600,
    };

    const result = await verify_onetime_payment_tx(
      JSON.stringify(config_json),
      dummyOneTimeCall.signature,
      dummyOneTimeCall.tx_hash
    );
    console.log(result);
  } catch (e) {
    console.error(e);
  }
}

async function main() {
  verifyPaymentChannelCall();
  verifyStreamsCall();
  verifyOneTimePayment();
}

main();
