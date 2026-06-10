mod adapters;
mod api;
mod batch;
mod circuit_breaker;
mod config;
mod engine;
mod idempotency;
mod middleware;
mod rates;
mod retry;
mod router;
mod store;
mod utils;
mod validation;
mod webhook;

use std::sync::Arc;

use axum::{middleware as axum_middleware, routing::{delete, get, post, put}, Router};
use tokio::net::TcpListener;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

use adapters::StellarAdapter;
use api::{AppContext, AppState};
use circuit_breaker::CircuitBreaker;
use config::{Config, Network};
use engine::Engine;
use rates::RateService;
use retry::RetryConfig;
use webhook::WebhookStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nodus_core_engine=info,tower_http=info".into()),
        )
        .init();

    let cfg = Config::from_env();

    let stellar_raw: Arc<dyn adapters::ChainAdapter> = match cfg.network {
        Network::Mainnet => {
            tracing::info!("network: Stellar Mainnet");
            Arc::new(StellarAdapter::mainnet())
        }
        Network::Testnet => {
            tracing::info!("network: Stellar Testnet");
            Arc::new(StellarAdapter::testnet())
        }
    };

    let stellar = Arc::new(CircuitBreaker::new(stellar_raw, 5, 30));

    let retry_config = RetryConfig::new(cfg.max_retry_attempts, cfg.retry_initial_delay_ms);
    let engine = Arc::new(Engine::new(vec![stellar], retry_config));
    let webhooks = Arc::new(WebhookStore::new());
    let rates = RateService::new();

    let state: AppState = Arc::new(AppContext { engine, rates, webhooks });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/healthz",                             get(api::health::healthz))
        .route("/api/v1/payments",                     post(api::payments::initiate).get(api::payments::list))
        .route("/api/v1/payments/simulate",            post(api::payments::simulate))
        .route("/api/v1/payments/batch",               post(api::batch::submit))
        .route("/api/v1/payments/:id",                 get(api::payments::get))
        .route("/api/v1/payments/:id/receipt",         get(api::payments::receipt))
        .route("/api/v1/fees/current",                 get(api::fees::current))
        .route("/api/v1/rates",                        get(api::rates::get))
        .route("/api/v1/webhooks",                     post(api::webhooks::register).get(api::webhooks::list))
        .route("/api/v1/webhooks/:id",                 delete(api::webhooks::delete))
        .route("/api/v1/webhooks/:id/toggle",          put(api::webhooks::toggle))
        .layer(axum_middleware::from_fn(middleware::inject_request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = TcpListener::bind(&addr).await.expect("failed to bind");
    tracing::info!("Nodus Protocol Core Engine listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
