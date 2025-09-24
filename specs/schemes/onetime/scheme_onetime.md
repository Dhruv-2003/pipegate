# Scheme: `onetime`

## Summary

One-time payments enable pay-per-use API access through direct blockchain transactions. This scheme allows consumers to make single payments to API providers and use transaction proof for subsequent API requests within a configured time window, perfect for occasional usage without ongoing commitments.

## Use Cases

- **Occasional API access**: Perfect for infrequent API usage without ongoing payment commitments or subscriptions
- **Resource downloads**: One-time payments for downloading datasets, research papers, reports, or large files
- **Trial access**: Limited-time access to premium API endpoints for evaluation or testing purposes
- **Simple micropayments**: Straightforward pay-per-call model without complex channel setup or streaming
- **Guest access**: Allow non-subscribers to access specific endpoints or features temporarily
- **Promotional usage**: Time-limited access for marketing campaigns, demonstrations, or special offers
- **Educational access**: Student or researcher access to expensive APIs for academic projects
- **Content purchases**: Pay-per-article, pay-per-report, or pay-per-dataset access models
- **Emergency access**: Quick access to critical APIs when regular payment methods are unavailable

## Appendix

One-time payments require direct blockchain transactions to provider addresses. The scheme supports any ERC-20 token transfer and operates on all EVM-compatible networks. Session management includes configurable time windows, TTL periods, and redemption limits to prevent payment abuse while allowing reasonable reuse of payments.
