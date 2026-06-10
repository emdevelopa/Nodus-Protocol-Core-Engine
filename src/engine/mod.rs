use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use crate::adapters::ChainAdapter;
use crate::utils::{now_utc, EngineError, FeeEstimate, Payment, PaymentStatus, Urgency};

// ── Engine ───────────────────────────────────────────────────────────────────

pub struct Engine {
    adapter: Arc<dyn ChainAdapter>,
    payments: DashMap<String, Payment>,
}

impl Engine {
    pub fn new(adapter: Arc<dyn ChainAdapter>) -> Self {
        Self {
            adapter,
            payments: DashMap::new(),
        }
    }

    // ── Payment lifecycle ─────────────────────────────────────────────────

    pub async fn initiate(
        &self,
        sender: String,
        recipient: String,
        amount: u64,
        token: String,
        urgency: Urgency,
    ) -> Result<Payment, EngineError> {
        if sender.is_empty() || recipient.is_empty() {
            return Err(EngineError::InvalidRequest(
                "sender and recipient are required".into(),
            ));
        }
        if amount == 0 {
            return Err(EngineError::InvalidRequest("amount must be > 0".into()));
        }

        let fees = self.adapter.fee_estimate().await.unwrap_or_default();
        let fee_stroops = match urgency {
            Urgency::Standard => fees.standard_stroops,
            Urgency::Fast => fees.fast_stroops,
            Urgency::Urgent => fees.urgent_stroops,
        };

        let now = now_utc();
        let mut payment = Payment {
            id: Uuid::new_v4().to_string(),
            sender,
            recipient,
            amount,
            token,
            status: PaymentStatus::Pending,
            tx_hash: None,
            fee_stroops,
            urgency,
            error: None,
            created_at: now.clone(),
            updated_at: now,
        };

        self.payments.insert(payment.id.clone(), payment.clone());
        tracing::info!(payment_id = %payment.id, "payment created");

        // Submit to chain asynchronously; update status in place.
        payment.status = PaymentStatus::Processing;
        self.payments.insert(payment.id.clone(), payment.clone());

        match self.adapter.submit(&payment).await {
            Ok(tx_hash) => {
                payment.status = PaymentStatus::Confirmed;
                payment.tx_hash = Some(tx_hash);
                payment.updated_at = now_utc();
                tracing::info!(
                    payment_id = %payment.id,
                    tx_hash    = ?payment.tx_hash,
                    "payment confirmed"
                );
            }
            Err(e) => {
                tracing::warn!(payment_id = %payment.id, error = %e, "payment failed");
                payment.status = PaymentStatus::Failed;
                payment.error = Some(e.to_string());
                payment.updated_at = now_utc();
            }
        }

        self.payments.insert(payment.id.clone(), payment.clone());
        Ok(payment)
    }

    pub fn get(&self, id: &str) -> Result<Payment, EngineError> {
        self.payments
            .get(id)
            .map(|r| r.value().clone())
            .ok_or_else(|| EngineError::NotFound(id.to_string()))
    }

    pub fn list(&self) -> Vec<Payment> {
        self.payments.iter().map(|r| r.value().clone()).collect()
    }

    // ── Simulation (dry-run) ──────────────────────────────────────────────

    pub async fn simulate(
        &self,
        sender: String,
        recipient: String,
        amount: u64,
        token: String,
        urgency: Urgency,
    ) -> Result<SimulationResult, EngineError> {
        if amount == 0 {
            return Err(EngineError::InvalidRequest("amount must be > 0".into()));
        }

        let fees = self.adapter.fee_estimate().await.unwrap_or_default();
        let fee_stroops = match urgency {
            Urgency::Standard => fees.standard_stroops,
            Urgency::Fast => fees.fast_stroops,
            Urgency::Urgent => fees.urgent_stroops,
        };
        let estimated_seconds = match urgency {
            Urgency::Standard => fees.standard_seconds,
            Urgency::Fast => fees.fast_seconds,
            Urgency::Urgent => fees.urgent_seconds,
        };

        Ok(SimulationResult {
            sender,
            recipient,
            amount,
            token,
            fee_stroops,
            chain: self.adapter.name().to_string(),
            estimated_confirmation_seconds: estimated_seconds,
        })
    }

    // ── Fees ──────────────────────────────────────────────────────────────

    pub async fn current_fees(&self) -> FeeEstimate {
        self.adapter.fee_estimate().await.unwrap_or_default()
    }

    // ── Health ────────────────────────────────────────────────────────────

    pub async fn health(&self) -> HealthStatus {
        let chain_ok = self.adapter.fee_estimate().await.is_ok();
        HealthStatus {
            status: if chain_ok { "ok" } else { "degraded" },
            chain: self.adapter.name(),
            chain_reachable: chain_ok,
            payments_in_store: self.payments.len(),
        }
    }
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct SimulationResult {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub token: String,
    pub fee_stroops: u64,
    pub chain: String,
    pub estimated_confirmation_seconds: u32,
}

#[derive(serde::Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub chain: &'static str,
    pub chain_reachable: bool,
    pub payments_in_store: usize,
}
