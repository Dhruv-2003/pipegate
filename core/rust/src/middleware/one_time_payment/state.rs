use alloy::primitives::FixedBytes;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::middleware::one_time_payment::types::OneTimePayment;

#[derive(Clone)]
pub struct OneTimePaymentState {
    // Transaction hash to payment mapping (tx_hash is the identifier)
    payments: Arc<RwLock<HashMap<FixedBytes<32>, OneTimePayment>>>,
}

impl OneTimePaymentState {
    pub fn new() -> Self {
        Self {
            payments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, tx_hash: FixedBytes<32>) -> Option<OneTimePayment> {
        let payments = self.payments.read().await;
        payments.get(&tx_hash).cloned()
    }

    pub async fn set(&self, tx_hash: FixedBytes<32>, payment: OneTimePayment) {
        let mut payments = self.payments.write().await;
        payments.insert(tx_hash, payment);
    }

    pub async fn invalidate(&self, tx_hash: FixedBytes<32>) {
        let mut payments = self.payments.write().await;
        payments.remove(&tx_hash);
    }

    // Additional helper methods for one-time payment logic
    pub async fn increment_redemptions(&self, tx_hash: FixedBytes<32>) -> Option<u32> {
        let mut payments = self.payments.write().await;
        if let Some(payment) = payments.get_mut(&tx_hash) {
            payment.redemptions += 1;
            Some(payment.redemptions)
        } else {
            None
        }
    }

    pub async fn set_first_redeemed(&self, tx_hash: FixedBytes<32>, timestamp: u64) -> bool {
        let mut payments = self.payments.write().await;
        if let Some(payment) = payments.get_mut(&tx_hash) {
            if payment.first_reedemed == 0 {
                payment.first_reedemed = timestamp;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub async fn is_valid_for_redemption(
        &self,
        tx_hash: FixedBytes<32>,
        current_time: u64,
    ) -> bool {
        self.is_valid_for_redemption_with_period(tx_hash, current_time, None, None).await
    }

    pub async fn is_valid_for_redemption_with_period(
        &self,
        tx_hash: FixedBytes<32>,
        current_time: u64,
        custom_period: Option<u64>, // Custom absolute window period
        custom_session_ttl: Option<u64>, // Custom session TTL
    ) -> bool {
        let payments = self.payments.read().await;
        if let Some(payment) = payments.get(&tx_hash) {
            // Check if payment hasn't exceeded max redemptions
            if payment.redemptions >= super::types::MAX_REDEMPTIONS {
                return false;
            }

            // Use custom period if provided, otherwise fallback to hardcoded value
            let abs_window = custom_period.unwrap_or(super::types::ABS_WINDOW_SEC);
            
            // Check if payment is within the absolute window from payment timestamp
            if current_time > payment.payment_timestamp + abs_window {
                return false;
            }

            // If this is the first redemption, it's valid
            if payment.first_reedemed == 0 {
                return true;
            }

            // Use custom session TTL if provided, otherwise fallback to hardcoded value
            let session_ttl = custom_session_ttl.unwrap_or(super::types::SESSION_TTL_SEC);
            
            // Check if we're within the session TTL from first redemption
            current_time <= payment.first_reedemed + session_ttl
        } else {
            false
        }
    }
}
