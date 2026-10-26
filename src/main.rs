use alertmanager_ntfy_adapter::config::Config;
use alertmanager_ntfy_adapter::create_router;
use alertmanager_ntfy_adapter::handler::AppState;
use alertmanager_ntfy_adapter::ntfy::NtfyClient;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,alertmanager_ntfy_adapter=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    info!(
        listen = %config.listen_addr,
        ntfy_url = %config.ntfy_url,
        ntfy_topic = %config.ntfy_topic,
        has_token = config.ntfy_token.is_some(),
        "Starting alertmanager-ntfy-adapter"
    );

    let ntfy_client = Arc::new(NtfyClient::new(&config));
    let state = Arc::new(AppState {
        config: config.clone(),
        ntfy_client,
    });

    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    info!("Server listening on {}", config.listen_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT, initiating shutdown..."),
        _ = terminate => info!("Received SIGTERM, initiating shutdown..."),
    }
}
