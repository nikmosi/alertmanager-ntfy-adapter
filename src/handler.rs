use crate::formatter::format_notification;
use crate::ntfy::NtfyClient;
use crate::types::AlertmanagerPayload;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use tracing::{error, info};

pub struct AppState {
    pub config: crate::config::Config,
    pub ntfy_client: Arc<NtfyClient>,
}

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok\n")
}

pub async fn handle_alertmanager(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AlertmanagerPayload>,
) -> Response {
    let alert_count = payload.alerts.len();
    info!(
        status = %payload.status,
        alerts = alert_count,
        receiver = payload.receiver.as_deref().unwrap_or("none"),
        "Received Alertmanager webhook"
    );

    let ntfy_payload = format_notification(&state.config, &payload);

    match state.ntfy_client.publish(&ntfy_payload).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            error!(error = %err, "Failed to deliver alert to ntfy");
            (StatusCode::BAD_GATEWAY, err.to_string()).into_response()
        }
    }
}
