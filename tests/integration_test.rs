use alertmanager_ntfy_adapter::config::Config;
use alertmanager_ntfy_adapter::create_router;
use alertmanager_ntfy_adapter::handler::AppState;
use alertmanager_ntfy_adapter::ntfy::NtfyClient;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let config = Config {
        ntfy_url: "http://127.0.0.1:9".to_string(), // Unreachable for test
        ntfy_topic: "test-topic".to_string(),
        ntfy_token: None,
        grafana_url: "https://grafana.test".to_string(),
        grafana_datasource_uid: "prometheus".to_string(),
        grafana_org_id: "1".to_string(),
        grafana_range_from: "now-1h".to_string(),
        grafana_range_to: "now".to_string(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
    };
    let ntfy_client = Arc::new(NtfyClient::new(&config));
    let state = Arc::new(AppState {
        config,
        ntfy_client,
    });
    create_router(state)
}

#[tokio::test]
async fn test_healthz_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok\n");
}

#[tokio::test]
async fn test_alertmanager_unreachable_upstream_returns_502() {
    let app = test_app();

    let payload = r#"{
        "version": "4",
        "status": "firing",
        "alerts": [
            {
                "status": "firing",
                "labels": {"alertname": "TestProbe", "severity": "warning"},
                "annotations": {"summary": "A test probe alert"}
            }
        ]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/alertmanager")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}
