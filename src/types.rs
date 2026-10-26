use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Webhook payload sent by Prometheus Alertmanager.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AlertmanagerPayload {
    pub version: Option<String>,
    #[serde(rename = "groupKey")]
    pub group_key: Option<String>,
    #[serde(rename = "truncatedAlerts", default)]
    pub truncated_alerts: u64,
    #[serde(default = "default_status")]
    pub status: String,
    pub receiver: Option<String>,
    #[serde(default)]
    pub alerts: Vec<Alert>,
    #[serde(rename = "groupLabels", default)]
    pub group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels", default)]
    pub common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations", default)]
    pub common_annotations: HashMap<String, String>,
    #[serde(rename = "externalURL")]
    pub external_url: Option<String>,
}

fn default_status() -> String {
    "firing".to_string()
}

/// An individual alert instance in Alertmanager.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Alert {
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    pub starts_at: Option<String>,
    #[serde(rename = "endsAt")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL")]
    pub generator_url: Option<String>,
    pub fingerprint: Option<String>,
}

impl Alert {
    pub fn label<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.labels.get(key).map(|s| s.as_str()).unwrap_or(default)
    }

    pub fn annotation<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.annotations
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
    }
}

/// Outgoing notification payload expected by ntfy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NtfyPayload {
    pub topic: String,
    pub title: String,
    pub message: String,
    pub priority: u8,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<String>,
}
