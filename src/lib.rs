pub mod config;
pub mod formatter;
pub mod handler;
pub mod ntfy;
pub mod types;

use axum::routing::{get, post};
use axum::Router;
use handler::{handle_alertmanager, healthz, AppState};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/alertmanager", post(handle_alertmanager))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
