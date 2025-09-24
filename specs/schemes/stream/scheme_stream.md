# Scheme: `stream`

## Summary

Stream payments enable continuous, real-time micropayments for subscription-based APIs through money streaming protocols. This scheme allows consumers to create ongoing payment flows that continuously stream tokens to API providers, enabling pay-per-second usage models without individual transactions.

## Use Cases

- **Subscription-based APIs**: Pay continuously for ongoing API access with automatic billing
- **Real-time data feeds**: Stream payments for live market data, news feeds, or analytics
- **Usage-based billing**: Automatic payment adjustment based on actual API consumption
- **Long-running processes**: Pay for extended computational, AI processing, or storage services
- **Content streaming**: Micropayments for media streaming, podcasts, or live content
- **IoT data streams**: Continuous payments for sensor data, telemetry, and device monitoring
- **Professional services**: Real-time payments for consulting, development, or support services
- **Research access**: Pay-per-second access to expensive datasets or computational resources

## Appendix

Stream payments require integration with money streaming protocols like Superfluid. The scheme supports any streaming-compatible token and operates on networks with streaming infrastructure. State management combines on-chain authoritative data with performance-optimized local caching for sub-50ms verification times.
