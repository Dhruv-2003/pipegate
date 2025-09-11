# Changelog

## [V0.6.0] - 2025-??-?? (UNRELEASED)

### Added

- Unified multi-scheme payments middleware: `PaymentsLayer` (`PipegateMiddlewareLayer` alias) and `Payments<S>` (`PipegateMiddleware<S>` alias) handling One-Time, Superfluid Streams, and Payment Channels in a single Tower layer.
- Public aliases `PaymentsState` and `PaymentsConfig` for ergonomic naming.

### Deprecated

- Per-scheme middleware layers/services in favor of unified layer:
  - `OnetimePaymentMiddlewareLayer` / `OnetimePaymentMiddleware<S>` (alias `OneTimePaymentMiddlewareLayer`) → use `PaymentsLayer` / `Payments<S>`
  - `StreamMiddlewareLayer` / `StreamMiddleware<S>` → use `PaymentsLayer` / `Payments<S>`
  - `PaymentChannelMiddlewareLayer` / `PaymentChannelMiddleware<S>` → use `PaymentsLayer` / `Payments<S>`
- These deprecated items will be removed in a future major release after a grace period.

### Changed

- Internal verification logic consolidated; header parsing now routes by `Scheme` enum and produces unified x402 responses.

### Migration Guide

1. Replace any per-scheme middleware layers with a single `PaymentsLayer::new(state, config)` where `config` contains scheme configurations.
2. Remove multiple middleware registrations—only one unified layer is required.
3. Keep using existing state initialization helpers; the unified layer lazily initializes per-scheme state as needed.
4. Update import paths to prefer `middleware::PaymentsLayer` and `middleware::Payments`.

### Notes

- Deprecated symbols continue to function but emit `#[deprecated]` warnings.
- No breaking changes introduced in 0.6.0; removal scheduled for the next semver-major.

## [V0.5.0] - 2025-02-22

### Added

- Introduced Tower-based Services for payment verification:
  - `OnetimePaymentMiddlewareLayer` for one-time payments
  - `StreamMiddlewareLayer` for stream-based payments
- Implemented cache system for stream verification to improve latency
- Added new components for stream-based payments:
  - `StreamListener` for cache invalidation on stream modifications
  - `StreamVerifier` for WASM compatibility with built-in caching
- Re-exported essential Alloy types for enhanced usability

### Changed

- Completed major codebase restructuring for improved maintainability
- Renamed middleware components for clarity:
  - `PipegateMiddlewareLayer` → `PaymentChannelMiddlewareLayer`
- Enhanced `PaymentChannelMiddlewareLayer` configuration:
  - Now accepts comprehensive `PaymentChannelConfig` instead of simple amount parameter

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
