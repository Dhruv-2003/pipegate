# Changelog

## [V0.6.0] - 2025-09-16

### Added

- `withPaymentInterceptor`: Unified function supporting all payment schemes (one-time, streams, channels)
- x402 standard compliance with standardized `X-Payment` header format
- Automatic 402 Payment Required handling and retry logic

### Breaking Changes

- Legacy per-scheme interceptors are now deprecated
- Payment header format updated to x402 standard

### Deprecated

- `createOneTimePaymentRequestInterceptor` - use `withPaymentInterceptor` with `oneTimePaymentTxHash`
- `createStreamRequestInterceptor` - use `withPaymentInterceptor` with `streamSender`
- `createPaymentChannelRequestInterceptor` - use `withPaymentInterceptor` with `channel`

## [V0.5.1] - 2025-03-18

Changes:

- Separated the build for ESM and CJS, No import changes required

## Removed

- WASM is un-packaged from the SDK - making it lighter by 1.4 mb , now it is required to be imported separately from the CDN

## [V0.5.0] - 2025-02-22

Breaking Changes:

- Updated WASM file export implementation, which may impact compatibility with previous versions
- TypeScript SDK now supports both client and server-side integrations through enhanced WASM functions

### Added

- Introduced new WASM functions supporting both streaming and one-time verification
- Implemented `StreamVerifier` component with caching support for WASM-based stream verification
