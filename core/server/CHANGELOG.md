# Changelog

## [V0.4.0] - 2024-12-26

### Added

- Introduced new one time payment channel verification methods
- Added support for WASM.
- Implemented helper functions for support other than axum.

### Changed

- Updated the middleware from a fn to Tower Service for extra compatibility.
- Improved error handling & detailed messages in the API.

### Removed

- Removed auth_middleware function to be directly used with a middleware.

## [V0.3.0] - 2024-12-09

### Added

- Initial release with auth_middleware function.
