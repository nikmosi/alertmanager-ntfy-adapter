use crate::config::Config;
use crate::types::NtfyPayload;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::fmt;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug)]
pub enum NtfyError {
    Request(reqwest::Error),
    BadStatus { status: u16, body: String },
}

impl fmt::Display for NtfyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NtfyError::Request(e) => write!(f, "HTTP request error: {e}"),
            NtfyError::BadStatus { status, body } => {
                write!(f, "ntfy returned HTTP status {status}: {body}")
            }
        }
    }
}

impl std::error::Error for NtfyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NtfyError::Request(e) => Some(e),
            NtfyError::BadStatus { .. } => None,
        }
    }
}

impl From<reqwest::Error> for NtfyError {
    fn from(err: reqwest::Error) -> Self {
        NtfyError::Request(err)
    }
}

pub struct NtfyClient {
    client: reqwest::Client,
    url: String,
    token: Option<String>,
}

impl NtfyClient {
    pub fn new(config: &Config) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(default_headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            url: config.ntfy_url.clone(),
            token: config.ntfy_token.clone(),
        }
    }

    pub async fn publish(&self, payload: &NtfyPayload) -> Result<(), NtfyError> {
        let mut req = self.client.post(&self.url).json(payload);

        if let Some(token) = &self.token {
            let auth_val = format!("Bearer {token}");
            if let Ok(header) = HeaderValue::from_str(&auth_val) {
                req = req.header(AUTHORIZATION, header);
            }
        }

        debug!(url = %self.url, topic = %payload.topic, "Publishing to ntfy");

        let response = req.send().await?;
        let status = response.status();

        if status.is_success() {
            info!(
                status = %status,
                topic = %payload.topic,
                title = %payload.title,
                "Notification published successfully to ntfy"
            );
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                body = %body,
                "ntfy rejected notification"
            );
            Err(NtfyError::BadStatus {
                status: status.as_u16(),
                body,
            })
        }
    }
}
