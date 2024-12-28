import init, {
  PaymentChannelVerifier,
  initialize_logging,
} from "../src/wasm/pipegate.js";
// import fs from "fs";

const dummyPaymentChannel = {
  address: "0xC72DfAC1a7B3Bc178F10Dc3bf36c7F64cf41B7DE",
  sender: "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33",
  recipient: "0x62C43323447899acb61C18181e34168903E033Bf",
  balance: "1000000",
  nonce: "0",
  expiration: "1736528914",
  channel_id: "2",
};

const dummyData = {
  message: "0xf1e267d9580d52e784d9681653dd37ad9674780da936fcd265e46e10d333b42e",
  signature:
    "0x4de2d31f6bd2aab04725c3e454c2d4b6fc5485c920e66465d55894c25d7c9688005571179ee04c1570c5740712ca6e7a428da04c91270cad1aa9671dc17ba28b1c",
  paymentChannelJSON: JSON.stringify(dummyPaymentChannel),
  paymentAmount: BigInt(1000),
  timestamp: BigInt(1735387288),
  bodyBytes: new Uint8Array(0),
};

async function main() {
  try {
    await init().catch(async (e) => {
      console.warn("Async initialization failed, trying sync...", e);
      // If async fails, try sync initialization
      // initSync();
    });

    initialize_logging();

    const rpc_url = "https://base-sepolia-rpc.publicnode.com";

    // @ts-ignore
    const verifier = new PaymentChannelVerifier(rpc_url);

    const updatedChannel = await verifier.verify_request(
      dummyData.message,
      dummyData.signature,
      dummyData.paymentChannelJSON,
      dummyData.paymentAmount,
      dummyData.timestamp,
      dummyData.bodyBytes
    );

    console.log(updatedChannel);
  } catch (e) {
    console.error(e);
  }
}

main();
