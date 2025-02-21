# Changelog

## [V0.5.0] - 2025-02-22

### Added

- Onetime and Streaming middleware functions converted to Tower Services and now exported as `OneTimePaymentMiddlewareLayer` and `StreamingPaymentMiddlewareLayer` respectively.
- Added `StreamVerifier` for WASM support of stream based verifications with support for cache
- Cache support added for stream based verification
- Added `StreamListener` for optimisation in stream based verification to listen to stream modifications and invalidate the cache.
- Alloy required types re-exported for better usability.

### Changed

- Major refactor in the codebase for better readability and maintainability, so imports might change.
- `PipegateMiddlewareLayer` now renamed to `PaymentChannelMiddlewareLayer` for better naming
- `PaymentChannelMiddlewareLayer` now accepts a `PaymentChannelConfig` instead of `amount` for better control over the channel.

## [V0.4.1] - 2024-12-28

### Added

- Middleware function for one time payment verification with axum. Example added in README.

### Changed

- Updated the headers parsing helper functions to be more generic and not accept `HeaderMap` instead of the whole request.
- Timestamp added in signed request

### Removed

- `auth_middleware` function removed from the crate.

## [V0.4.0] - 2024-12-26

### Added

- Introduced new one time payment verification methods
- Added support for WASM.
- Implemented helper functions for supporting other frameworks than axum.

### Changed

- Updated the middleware from a fn to Tower Service for extra compatibility.
- Improved error handling & detailed messages in the API.

### Removed

- Removed auth_middleware function to be directly used with a middleware.

## [V0.3.0] - 2024-12-09

### Added

- Initial release with auth_middleware function.
