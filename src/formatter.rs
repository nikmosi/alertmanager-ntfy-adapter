use crate::config::Config;
use crate::types::{Alert, AlertmanagerPayload, NtfyPayload};
use std::collections::BTreeSet;
use url::Url;

pub fn alert_name<'a>(alert: &'a Alert) -> &'a str {
    alert.label("alertname", "Alert")
}

pub fn alert_subject(alert: &Alert) -> String {
    let namespace = alert.label("namespace", "");
    let pod = alert.label("pod", "");
    let container = alert.label("container", "");

    if !namespace.is_empty() && !pod.is_empty() && !container.is_empty() {
        return format!("{}/{} ({})", namespace, pod, container);
    }
    if !namespace.is_empty() && !pod.is_empty() {
        return format!("{}/{}", namespace, pod);
    }
    alert.label("instance", "").to_string()
}

pub fn priority(status: &str, alerts: &[Alert]) -> u8 {
    if status.eq_ignore_ascii_case("resolved") {
        return 2;
    }
    let severities: BTreeSet<&str> = alerts
        .iter()
        .map(|a| a.label("severity", ""))
        .filter(|s| !s.is_empty())
        .collect();

    if severities.contains("critical") {
        5
    } else if severities.contains("warning") {
        4
    } else {
        3
    }
}

pub fn tags(status: &str, alerts: &[Alert]) -> Vec<String> {
    let severities: BTreeSet<&str> = alerts
        .iter()
        .map(|a| a.label("severity", ""))
        .filter(|s| !s.is_empty())
        .collect();

    let mut result = vec!["prometheus".to_string(), "alertmanager".to_string()];
    for sev in severities {
        result.push(sev.to_string());
    }

    if status.eq_ignore_ascii_case("resolved") {
        result.push("white_check_mark".to_string());
    } else {
        result.push("rotating_light".to_string());
    }

    result
}

pub fn title(status: &str, alerts: &[Alert]) -> String {
    let first = alerts.first();
    let name = first.map(alert_name).unwrap_or("Alert");
    let suffix = first.map(alert_subject).unwrap_or_default();
    let status_text = status.to_ascii_uppercase();
    let count = alerts.len();
    let base = format!("[{status_text}:{count}] {name}");

    if !suffix.is_empty() {
        format!("{base} {suffix}")
    } else {
        base
    }
}

pub fn message(alerts: &[Alert]) -> String {
    let mut parts = Vec::new();

    for alert in alerts {
        let mut lines = Vec::new();
        let summary = alert.annotation("summary", "");
        let description = alert.annotation("description", "");

        if !summary.is_empty() {
            lines.push(summary.to_string());
        }

        let subject = alert_subject(alert);
        if !subject.is_empty() {
            lines.push(format!("Target: {subject}"));
        }

        let severity = alert.label("severity", "");
        if !severity.is_empty() {
            lines.push(format!("Severity: {severity}"));
        }

        let reason = alert.label("reason", "");
        if !reason.is_empty() {
            lines.push(format!("Reason: {reason}"));
        }

        if !description.is_empty() && description != summary {
            lines.push(description.to_string());
        }

        let runbook = alert.annotation("runbook_url", "");
        if !runbook.is_empty() {
            lines.push(format!("Runbook: {runbook}"));
        }

        if lines.is_empty() {
            parts.push(alert_name(alert).to_string());
        } else {
            parts.push(lines.join("\n"));
        }
    }

    parts.join("\n\n---\n\n")
}

pub fn prometheus_expr(generator_url: &str) -> Option<String> {
    let url = Url::parse(generator_url).ok()?;
    for (key, val) in url.query_pairs() {
        if key == "g0.expr" || key == "query" || key == "expr" {
            if !val.is_empty() {
                return Some(val.into_owned());
            }
        }
    }
    None
}

pub fn grafana_explore_url(config: &Config, expr: &str) -> String {
    let pane = serde_json::json!({
        "default": {
            "datasource": config.grafana_datasource_uid,
            "queries": [
                {
                    "refId": "A",
                    "datasource": {
                        "uid": config.grafana_datasource_uid,
                        "type": "prometheus"
                    },
                    "expr": expr,
                    "range": true
                }
            ],
            "range": {
                "from": config.grafana_range_from,
                "to": config.grafana_range_to
            }
        }
    });

    let pane_str = serde_json::to_string(&pane).unwrap_or_default();
    let encoded_panes: String = url::form_urlencoded::byte_serialize(pane_str.as_bytes()).collect();

    format!(
        "{}/explore?panes={}&schemaVersion=1&orgId={}",
        config.grafana_url, encoded_panes, config.grafana_org_id
    )
}

pub fn click(config: &Config, alerts: &[Alert]) -> String {
    let first = match alerts.first() {
        Some(a) => a,
        None => return format!("{}/alerting/list", config.grafana_url),
    };

    let explicit_url = first.annotation("grafana_url", "");
    if !explicit_url.is_empty() {
        return explicit_url.to_string();
    }

    if let Some(generator_url) = first.generator_url.as_deref() {
        if let Some(expr) = prometheus_expr(generator_url) {
            return grafana_explore_url(config, &expr);
        }
    }

    let runbook = first.annotation("runbook_url", "");
    if !runbook.is_empty() {
        return runbook.to_string();
    }

    format!("{}/alerting/list", config.grafana_url)
}

pub fn format_notification(config: &Config, payload: &AlertmanagerPayload) -> NtfyPayload {
    let alerts = &payload.alerts;
    let status = payload.status.as_str();

    let click_url = click(config, alerts);
    let click_option = if click_url.is_empty() {
        None
    } else {
        Some(click_url)
    };

    NtfyPayload {
        topic: config.ntfy_topic.clone(),
        title: title(status, alerts),
        message: message(alerts),
        priority: priority(status, alerts),
        tags: tags(status, alerts),
        click: click_option,
    }
}
