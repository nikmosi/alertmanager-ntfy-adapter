use alertmanager_ntfy_adapter::config::Config;
use alertmanager_ntfy_adapter::formatter::*;
use alertmanager_ntfy_adapter::types::{Alert, AlertmanagerPayload};
use std::collections::HashMap;

fn mock_config() -> Config {
    Config {
        ntfy_url: "http://ntfy.apps.svc.cluster.local/".to_string(),
        ntfy_topic: "grafana".to_string(),
        ntfy_token: Some("secret-token".to_string()),
        grafana_url: "https://grafana.xinfra.ru".to_string(),
        grafana_datasource_uid: "prometheus".to_string(),
        grafana_org_id: "1".to_string(),
        grafana_range_from: "now-1h".to_string(),
        grafana_range_to: "now".to_string(),
        listen_addr: "0.0.0.0:8080".parse().unwrap(),
    }
}

fn alert(name: &str, severity: &str, instance: &str, summary: &str) -> Alert {
    let mut labels = HashMap::new();
    labels.insert("alertname".to_string(), name.to_string());
    if !severity.is_empty() {
        labels.insert("severity".to_string(), severity.to_string());
    }
    if !instance.is_empty() {
        labels.insert("instance".to_string(), instance.to_string());
    }

    let mut annotations = HashMap::new();
    if !summary.is_empty() {
        annotations.insert("summary".to_string(), summary.to_string());
    }

    Alert {
        status: "firing".to_string(),
        labels,
        annotations,
        starts_at: Some("2026-10-26T12:00:00Z".to_string()),
        ends_at: None,
        generator_url: None,
        fingerprint: None,
    }
}

#[test]
fn test_subject_hierarchy() {
    let mut a = alert("KubePodCrashLooping", "critical", "node-1", "Pod crash");
    assert_eq!(alert_subject(&a), "node-1");

    a.labels
        .insert("namespace".to_string(), "monitoring".to_string());
    a.labels.insert("pod".to_string(), "prom-0".to_string());
    assert_eq!(alert_subject(&a), "monitoring/prom-0");

    a.labels
        .insert("container".to_string(), "prometheus".to_string());
    assert_eq!(alert_subject(&a), "monitoring/prom-0 (prometheus)");
}

#[test]
fn test_title_formatting() {
    let mut a1 = alert("VpsNodeDown", "critical", "vpn-timeweb", "VPS is down");
    let a2 = alert("VpsNodeDown", "critical", "vpn-finka", "VPS is down");

    let alerts = vec![a1.clone(), a2];
    assert_eq!(title("firing", &alerts), "[FIRING:2] VpsNodeDown vpn-timeweb");
    assert_eq!(title("resolved", &alerts), "[RESOLVED:2] VpsNodeDown vpn-timeweb");

    a1.labels.remove("instance");
    let single = vec![a1];
    assert_eq!(title("firing", &single), "[FIRING:1] VpsNodeDown");
}

#[test]
fn test_priority_calculation() {
    let a_crit = alert("A", "critical", "", "");
    let a_warn = alert("B", "warning", "", "");
    let a_info = alert("C", "info", "", "");

    assert_eq!(priority("resolved", &[a_crit.clone()]), 2);
    assert_eq!(priority("firing", &[a_crit, a_warn.clone()]), 5);
    assert_eq!(priority("firing", &[a_warn, a_info.clone()]), 4);
    assert_eq!(priority("firing", &[a_info]), 3);
    assert_eq!(priority("firing", &[]), 3);
}

#[test]
fn test_tags_formatting() {
    let a1 = alert("A", "critical", "", "");
    let a2 = alert("B", "warning", "", "");

    let firing_tags = tags("firing", &[a1.clone(), a2.clone()]);
    assert_eq!(
        firing_tags,
        vec![
            "prometheus".to_string(),
            "alertmanager".to_string(),
            "critical".to_string(),
            "warning".to_string(),
            "rotating_light".to_string()
        ]
    );

    let resolved_tags = tags("resolved", &[a1, a2]);
    assert_eq!(
        resolved_tags,
        vec![
            "prometheus".to_string(),
            "alertmanager".to_string(),
            "critical".to_string(),
            "warning".to_string(),
            "white_check_mark".to_string()
        ]
    );
}

#[test]
fn test_message_formatting() {
    let mut a1 = alert("VpsNodeDown", "critical", "vpn-timeweb", "Host unreachable");
    a1.annotations
        .insert("description".to_string(), "Ping dropped for > 2m".to_string());
    a1.annotations
        .insert("runbook_url".to_string(), "https://wiki.xinfra.ru/vps".to_string());

    let mut a2 = alert("VpsNodeDown", "critical", "vpn-finka", "Host unreachable");
    a2.labels.insert("reason".to_string(), "HostDown".to_string());

    let msg = message(&[a1, a2]);
    let expected = "\
Host unreachable
Target: vpn-timeweb
Severity: critical
Ping dropped for > 2m
Runbook: https://wiki.xinfra.ru/vps

---

Host unreachable
Target: vpn-finka
Severity: critical
Reason: HostDown";

    assert_eq!(msg, expected);
}

#[test]
fn test_generator_url_and_explore_link() {
    let cfg = mock_config();
    let url = "http://prometheus:9090/graph?g0.expr=up%7Bjob%3D%22node%22%7D+%3D%3D+0&g0.tab=1";
    let expr = prometheus_expr(url).expect("Should extract expr");
    assert_eq!(expr, "up{job=\"node\"} == 0");

    let explore = grafana_explore_url(&cfg, &expr);
    assert!(explore.starts_with("https://grafana.xinfra.ru/explore?panes="));
    assert!(explore.contains("schemaVersion=1&orgId=1"));
}

#[test]
fn test_click_hierarchy() {
    let cfg = mock_config();

    // 1. Explicit grafana_url annotation
    let mut a1 = alert("Test", "info", "", "");
    a1.annotations
        .insert("grafana_url".to_string(), "https://custom.grafana.com".to_string());
    assert_eq!(click(&cfg, &[a1.clone()]), "https://custom.grafana.com");

    // 2. Generator URL extraction
    a1.annotations.remove("grafana_url");
    a1.generator_url = Some("http://prom:9090/graph?expr=rate(cpu%5B5m%5D)".to_string());
    let click_url = click(&cfg, &[a1.clone()]);
    assert!(click_url.contains("/explore?panes="));

    // 3. Runbook URL
    a1.generator_url = None;
    a1.annotations
        .insert("runbook_url".to_string(), "https://wiki/runbook".to_string());
    assert_eq!(click(&cfg, &[a1.clone()]), "https://wiki/runbook");

    // 4. Fallback
    a1.annotations.remove("runbook_url");
    assert_eq!(click(&cfg, &[a1]), "https://grafana.xinfra.ru/alerting/list");
    assert_eq!(click(&cfg, &[]), "https://grafana.xinfra.ru/alerting/list");
}

#[test]
fn test_full_payload_formatting() {
    let cfg = mock_config();
    let payload = AlertmanagerPayload {
        version: Some("4".to_string()),
        status: "firing".to_string(),
        alerts: vec![alert("VpsNodeDown", "critical", "vpn-timeweb", "Lost telemetry")],
        ..Default::default()
    };

    let ntfy = format_notification(&cfg, &payload);
    assert_eq!(ntfy.topic, "grafana");
    assert_eq!(ntfy.title, "[FIRING:1] VpsNodeDown vpn-timeweb");
    assert_eq!(ntfy.priority, 5);
    assert_eq!(ntfy.tags, vec!["prometheus", "alertmanager", "critical", "rotating_light"]);
    assert_eq!(ntfy.click, Some("https://grafana.xinfra.ru/alerting/list".to_string()));
}
