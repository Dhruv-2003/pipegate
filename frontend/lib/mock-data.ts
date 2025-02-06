import { PaymentMethods } from "./types";

export const mockAPIData = [
  {
    id: "weather-api",
    name: "Weather API",
    shortDescription: "Real-time weather data for any location",
    fullDescription:
      "Get accurate weather forecasts, current conditions, and historical data for any location worldwide. Our API provides comprehensive meteorological information including temperature, humidity, wind speed, and more.",
    rating: 4.5,
    pricing: "Starting at $0.01 per call",
    paymentPlan: PaymentMethods.OnDemand,
    endpoints: [
      { method: "GET", path: "/current/{city}" },
      { method: "GET", path: "/forecast/{city}" },
      { method: "GET", path: "/historical/{city}/{date}" },
    ],
  },
  {
    id: "crypto-price-api",
    name: "Crypto Price API",
    shortDescription: "Real-time cryptocurrency price data",
    fullDescription:
      "Access up-to-the-minute pricing data for hundreds of cryptocurrencies. Our API provides real-time and historical price information, market cap data, and trading volume for major exchanges.",
    rating: 4.8,
    pricing: "Free tier available, premium plans start at $50/month",
    paymentPlan: PaymentMethods.Stream,
    endpoints: [
      { method: "GET", path: "/price/{coin}" },
      { method: "GET", path: "/marketcap/{coin}" },
      { method: "GET", path: "/exchange/{exchange}/volume" },
    ],
  },
  // Add more mock API data as needed
];

export const mockUserDashboardData = {
  activeSubscriptions: 3,
  totalAPICalls: 15234,
  subscriptions: [
    { apiName: "Weather API", status: "Active" },
    { apiName: "Crypto Price API", status: "Active" },
    { apiName: "Analytics API", status: "Inactive" },
  ],
  usageData: [
    { date: "2023-01-01", apiCalls: 500 },
    { date: "2023-01-02", apiCalls: 750 },
    { date: "2023-01-03", apiCalls: 600 },
    { date: "2023-01-04", apiCalls: 800 },
    { date: "2023-01-05", apiCalls: 950 },
  ],
};

export const mockProviderDashboardData = {
  totalUsers: 1250,
  totalRequests: 1000000,
  totalRevenue: 5000.0,
  revenueData: [
    { date: "2023-01-01", revenue: 150.0 },
    { date: "2023-01-02", revenue: 200.0 },
    { date: "2023-01-03", revenue: 180.0 },
    { date: "2023-01-04", revenue: 220.0 },
    { date: "2023-01-05", revenue: 250.0 },
  ],
  usageData: [
    { date: "2023-01-01", requests: 180000 },
    { date: "2023-01-02", requests: 195000 },
    { date: "2023-01-03", requests: 205000 },
    { date: "2023-01-04", requests: 220000 },
    { date: "2023-01-05", requests: 240000 },
  ],
};
