use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::engine::Engine;
use crate::rates::RateService;
use crate::utils::{ApiError, EngineError, Urgency};

pub type AppState = Arc<AppContext>;

pub struct AppContext {
    pub engine: Engine,
    pub rates: RateService,
}

impl IntoResponse for EngineError {
    fn into_response(self) -> Response {
        let status = match self.http_status() {
            404 => StatusCode::NOT_FOUND,
            400 => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let code = match &self {
            EngineError::NotFound(_)       => "NOT_FOUND",
            EngineError::InvalidRequest(_) => "INVALID_REQUEST",
            EngineError::AdapterError(_)   => "ADAPTER_ERROR",
            EngineError::NetworkError(_)   => "NETWORK_ERROR",
            EngineError::Internal(_)       => "INTERNAL_ERROR",
        };
        (status, Json(ApiError { code, message: self.to_string() })).into_response()
    }
}

pub async fn healthz(State(ctx): State<AppState>) -> impl IntoResponse {
    let health = ctx.engine.health().await;
    let status = if health.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(health))
}

#[derive(Debug, Deserialize)]
pub struct InitiatePaymentRequest {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub token: String,
    #[serde(default)]
    pub urgency: Urgency,
}

pub async fn initiate_payment(
    State(ctx): State<AppState>,
    Json(req): Json<InitiatePaymentRequest>,
) -> Result<(StatusCode, impl IntoResponse), EngineError> {
    let payment = ctx
        .engine
        .initiate(req.sender, req.recipient, req.amount, req.token, req.urgency)
        .await?;
    Ok((StatusCode::CREATED, Json(payment)))
}

pub async fn get_payment(
    State(ctx): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EngineError> {
    Ok(Json(ctx.engine.get(&id)?))
}

pub async fn list_payments(State(ctx): State<AppState>) -> impl IntoResponse {
    Json(ctx.engine.list())
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub token: String,
    #[serde(default)]
    pub urgency: Urgency,
}

pub async fn simulate_payment(
    State(ctx): State<AppState>,
    Json(req): Json<SimulateRequest>,
) -> Result<impl IntoResponse, EngineError> {
    let result = ctx
        .engine
        .simulate(req.sender, req.recipient, req.amount, req.token, req.urgency)
        .await?;
    Ok(Json(result))
}

pub async fn current_fees(State(ctx): State<AppState>) -> impl IntoResponse {
    Json(ctx.engine.current_fees().await)
}

#[derive(Serialize)]
pub struct Receipt {
    pub payment_id: String,
    pub tx_hash: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub token: String,
    pub chain: String,
    pub confirmed_at: String,
}

pub async fn get_receipt(
    State(ctx): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, EngineError> {
    let payment = ctx.engine.get(&id)?;
    let tx_hash = payment
        .tx_hash
        .ok_or_else(|| EngineError::InvalidRequest(format!("payment {id} is not confirmed")))?;

    Ok(Json(Receipt {
        payment_id: payment.id,
        tx_hash,
        sender: payment.sender,
        recipient: payment.recipient,
        amount: payment.amount,
        token: payment.token,
        chain: "stellar".into(),
        confirmed_at: payment.updated_at,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RatesQuery {
    pub tokens: Option<String>,
}

pub async fn get_rates(
    State(ctx): State<AppState>,
    Query(q): Query<RatesQuery>,
) -> impl IntoResponse {
    let tokens: Vec<&str> = q
        .tokens
        .as_deref()
        .unwrap_or("XLM,USDC")
        .split(',')
        .map(str::trim)
        .collect();
    Json(ctx.rates.rates_for(&tokens).await)
}
