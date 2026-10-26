use std::net::SocketAddr;
use std::path::Path;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct Config {
    pub ntfy_url: String,
    pub ntfy_topic: String,
    pub ntfy_token: Option<String>,
    pub grafana_url: String,
    pub grafana_datasource_uid: String,
    pub grafana_org_id: String,
    pub grafana_range_from: String,
    pub grafana_range_to: String,
    pub listen_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let ntfy_url = std::env::var("NTFY_URL")
            .unwrap_or_else(|_| "http://ntfy.apps.svc.cluster.local/".to_string());

        let ntfy_topic = std::env::var("NTFY_TOPIC").unwrap_or_else(|_| "grafana".to_string());

        let ntfy_token = Self::resolve_token();

        let grafana_url = std::env::var("GRAFANA_URL")
            .unwrap_or_else(|_| "https://grafana.xinfra.ru".to_string())
            .trim_end_matches('/')
            .to_string();

        let grafana_datasource_uid =
            std::env::var("GRAFANA_DATASOURCE_UID").unwrap_or_else(|_| "prometheus".to_string());

        let grafana_org_id = std::env::var("GRAFANA_ORG_ID").unwrap_or_else(|_| "1".to_string());

        let grafana_range_from =
            std::env::var("GRAFANA_RANGE_FROM").unwrap_or_else(|_| "now-1h".to_string());

        let grafana_range_to =
            std::env::var("GRAFANA_RANGE_TO").unwrap_or_else(|_| "now".to_string());

        let listen_addr: SocketAddr = std::env::var("LISTEN_ADDR")
            .or_else(|_| {
                std::env::var("PORT").map(|port| format!("0.0.0.0:{}", port))
            })
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .expect("Invalid LISTEN_ADDR or PORT configuration");

        Self {
            ntfy_url,
            ntfy_topic,
            ntfy_token,
            grafana_url,
            grafana_datasource_uid,
            grafana_org_id,
            grafana_range_from,
            grafana_range_to,
            listen_addr,
        }
    }

    fn resolve_token() -> Option<String> {
        if let Ok(token) = std::env::var("NTFY_TOKEN") {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        let token_path = std::env::var("NTFY_TOKEN_FILE")
            .unwrap_or_else(|_| "/etc/ntfy-token/token".to_string());

        if Path::new(&token_path).exists() {
            match std::fs::read_to_string(&token_path) {
                Ok(content) => {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
                Err(err) => {
                    warn!(path = %token_path, error = %err, "Failed to read ntfy token file");
                }
            }
        }

        None
    }
}
