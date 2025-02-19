use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::primitives::Address;
use tokio::sync::RwLock;

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use crate::error::AuthError;

#[derive(Clone)]
pub struct RateLimiter {
    rate_limiter: Arc<RwLock<HashMap<Address, (u64, SystemTime)>>>, // Rate limiter for the user
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, sender: Address) -> Result<(), AuthError> {
        const RATE_LIMIT: u64 = 100; // 100 requests
        const WINDOW: u64 = 60; // Every 60 seconds

        let mut rate_limits = self.rate_limiter.write().await;

        #[cfg(not(target_arch = "wasm32"))]
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        #[cfg(target_arch = "wasm32")]
        let now = (Date::now() as u64) / 1000;

        let (count, last_reset) = rate_limits.entry(sender).or_insert((0, SystemTime::now()));

        let last_reset_secs = last_reset.duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now - last_reset_secs >= WINDOW {
            *count = 1;
            *last_reset = SystemTime::now();
            Ok(())
        } else if *count >= RATE_LIMIT {
            Err(AuthError::RateLimitExceeded)
        } else {
            *count += 1;
            Ok(())
        }
    }
}
