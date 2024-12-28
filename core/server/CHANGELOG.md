# Changelog

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
