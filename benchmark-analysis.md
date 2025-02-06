# Benchmark analysis

- Signature Creation: 3-4 ms
- Message reconstruction & Signature verification: 2-3ms
- OnChain verification with 1 RPC call: 300-900 ms
- Other parsing headers and extraction, etc: 100 us

- Total middleware time: 305-905 ms (only for the first call, subsequent calls will be faster due to caching)
- Subsequent calls time (with caching): (30-50) ms

### **Comparison and Key Latency Breakdown:**

| **Method**                            | **Time for Auth (ms)** | **Time for Payment Verification (ms)** | **Total Time per Request (ms)** | **Setup & Cost**                      |
| ------------------------------------- | ---------------------- | -------------------------------------- | ------------------------------- | ------------------------------------- |
| **Pipegate Approach (Decentralized)** | 2-3                    | 300-900 (on-chain verification)        | 302-903                         | Simple & zero extra cost              |
| **Custom Auth (Internal DB)**         | 5-100                  | N/A                                    | 5-100                           | Complex & low to moderate cost        |
| **Stripe + Custom Backend**           | 50-200                 | 100-500 (Stripe API)                   | 150-700                         | Can be complex & 2-4% of total-amount |

---

### **Methods**

1. **Pipegate Approach:**

- **Latency**: The **signature verification** is super fast (2-3ms), but the **on-chain verification** is the main latency factor, with RPC call latencies between **300ms to 900ms**. This is still significantly faster than traditional methods, especially when using **payment channels** where you don't need to interact with a central authority or make repeated external calls.
- **Scalability**: This solution doesn’t rely on querying an external database or a third-party payment processor for every request, which scales much better under load. The main bottleneck could be the RPC latencies (which are dependent on the node you’re using), but you can work on mitigating that with better RPC services or by setting up your own node.

2. **Custom Auth + Internal DB**:

   **Process:**

   - The client includes an API key in each request.
   - The backend looks up the API key in its internal database.
   - It checks if the API key is valid, associated with an active user subscription, and allows access accordingly.
   - **Database Query**: The API key lookup requires a database query to verify subscription status, usage limits, etc. This can be relatively fast, depending on the database.

- **Latency**: The main limiting factor here is **database query time** which can take anywhere from **5ms to 100ms** depending on the database and load. If you're using an optimized in-memory store (like Redis), it could be much faster. But if you're querying a traditional relational database, the time could increase due to I/O operations.If you have an API gateway or load balancer, it could add an additional **5ms to 20ms**.
- **Scalability**: With high traffic, you'd need to ensure the database is properly indexed and can handle the request volume. Adding more API keys means scaling the database solution as well.**sharding** and **distributed databases** might be necessary, which can add complexity.

3. **Stripe + Backend**:

   **Process:**

   - The client makes a request to the API.
   - The backend uses Stripe's API to verify the user's payment status.
   - Every time a request is made, the backend queries Stripe to check if the user is still subscribed and has sufficient balance.
   - Stripe’s API handles the authentication, payments, and usage recording.

- **Latency**: Stripe’s API calls will inherently add external network latency to each request. Even with high-speed networks, this can be anywhere from **150-700ms**. This is a typical API call to an external service
- **Scalability**: This approach requires external payment processors to handle each request, and each call to Stripe is an external network call that could fail or be delayed. While Stripe handles large-scale payment services well, for microservices where you have high API call volumes, you might face rate limitations.

---

### **Optimisations:**

- **Caching**: The verified addresses are cacehd into an in memory store to reduce the verification time after the first call, reducing the latency to sub 50ms.
- **Listener**: To mitigate the issue of user cancelling or modifying the subscription, on chain event listener is attached that listens to the subscription events and updates the cache accordingly.
