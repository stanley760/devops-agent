use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

/// Prometheus 告警查询工具参数
#[derive(Deserialize)]
pub struct PrometheusAlertsArgs;

/// Prometheus 告警查询错误
#[derive(Debug, thiserror::Error)]
pub enum PrometheusError {
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("响应解析失败: {0}")]
    Parse(String),
}

/// 简化的告警信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplifiedAlert {
    pub alert_name: String,
    pub description: String,
    pub state: String,
    pub active_at: String,
    pub duration: String,
}

/// 告警查询输出
#[derive(Debug, Serialize)]
pub struct PrometheusAlertsOutput {
    pub success: bool,
    pub alerts: Vec<SimplifiedAlert>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Prometheus 告警查询工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusAlertsTool {
    pub base_url: String,
}

impl PrometheusAlertsTool {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl Tool for PrometheusAlertsTool {
    const NAME: &'static str = "query_prometheus_alerts";
    type Error = PrometheusError;
    type Args = PrometheusAlertsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "查询 Prometheus 当前活动的告警。返回所有 firing/pending 状态的告警，包含告警名称、描述、状态、激活时间和持续时间。当需要检查当前告警、调查告警条件或监控告警状态时使用此工具。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let url = format!("{}/api/v1/alerts", self.base_url);
        let resp: serde_json::Value = reqwest::get(&url)
            .await?
            .json()
            .await
            .map_err(|e| PrometheusError::Parse(e.to_string()))?;

        let simplified = simplify_alerts(&resp);
        let output = PrometheusAlertsOutput {
            success: true,
            alerts: simplified,
            message: "查询成功".to_string(),
            error: None,
        };

        Ok(serde_json::to_string_pretty(&output).unwrap_or_default())
    }
}

/// 从 Prometheus API 响应中提取并简化告警信息
/// 对相同 alertname 的告警只保留第一个
fn simplify_alerts(resp: &serde_json::Value) -> Vec<SimplifiedAlert> {
    let alerts = match resp.get("data").and_then(|d| d.get("alerts")) {
        Some(serde_json::Value::Array(alerts)) => alerts,
        _ => return vec![],
    };

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for alert in alerts {
        let labels = alert.get("labels").and_then(|l| l.as_object());
        let alert_name = labels
            .and_then(|l| l.get("alertname"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // 去重：相同 alertname 只保留第一个
        if seen.contains(&alert_name) {
            continue;
        }
        seen.insert(alert_name.clone());

        let annotations = alert.get("annotations").and_then(|a| a.as_object());
        let description = annotations
            .and_then(|a| a.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let state = alert
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let active_at = alert
            .get("activeAt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let duration = calculate_duration(&active_at);

        result.push(SimplifiedAlert {
            alert_name,
            description,
            state,
            active_at,
            duration,
        });
    }

    result
}

/// 计算从 activeAt 到现在的持续时间
fn calculate_duration(active_at: &str) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(active_at) else {
        return "unknown".to_string();
    };

    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(parsed);

    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_duration() {
        // 活跃时间在过去
        let past = (chrono::Utc::now() - chrono::Duration::hours(2) - chrono::Duration::minutes(30) - chrono::Duration::seconds(15))
            .to_rfc3339();
        let duration = calculate_duration(&past);
        assert!(duration.starts_with("2h30m"));
    }

    #[test]
    fn test_simplify_alerts_empty() {
        let resp = serde_json::json!({"status": "success", "data": {"alerts": []}});
        let result = simplify_alerts(&resp);
        assert!(result.is_empty());
    }

    #[test]
    fn test_simplify_alerts_dedup() {
        let resp = serde_json::json!({
            "data": {
                "alerts": [
                    {
                        "labels": {"alertname": "HighCPU"},
                        "annotations": {"description": "CPU is high"},
                        "state": "firing",
                        "activeAt": "2025-01-01T00:00:00Z"
                    },
                    {
                        "labels": {"alertname": "HighCPU"},
                        "annotations": {"description": "CPU is high on another instance"},
                        "state": "firing",
                        "activeAt": "2025-01-01T00:01:00Z"
                    },
                    {
                        "labels": {"alertname": "LowMemory"},
                        "annotations": {"description": "Memory is low"},
                        "state": "pending",
                        "activeAt": "2025-01-01T00:02:00Z"
                    }
                ]
            }
        });
        let result = simplify_alerts(&resp);
        assert_eq!(result.len(), 2); // HighCPU 去重后只保留一个
        assert_eq!(result[0].alert_name, "HighCPU");
        assert_eq!(result[1].alert_name, "LowMemory");
    }
}
