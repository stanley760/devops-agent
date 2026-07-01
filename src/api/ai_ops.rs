use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::agent::aiops_agent;
use crate::error::AppError;
use crate::pipeline::aiops::run_aiops_loop;
use crate::AppState;

/// AIOps 请求（当前为空，自动获取告警）
#[derive(Debug, Deserialize)]
pub struct AioPsRequest {}

/// AIOps 响应
#[derive(Debug, Serialize)]
pub struct AioPsResponse {
    /// 分析结论
    pub result: String,
    /// 详细步骤
    pub detail: Vec<String>,
}

/// POST /api/ai_ops — AIOps 自动告警分析
///
/// 执行 Plan-Execute-Replan 流程:
/// 1. 查询 Prometheus 获取当前告警
/// 2. 为每个告警制定调查计划
/// 3. 执行调查步骤
/// 4. 评估结果，必要时重新规划
/// 5. 生成最终报告
pub async fn ai_ops_handler(
    State(state): State<AppState>,
    Json(_req): Json<AioPsRequest>,
) -> Result<Json<AioPsResponse>, AppError> {
    tracing::info!("处理 AIOps 请求");

    // 构建 AIOps Agent 集合
    let agents = aiops_agent::build_aiops_agents(&state.config)
        .map_err(|e| AppError::Internal(format!("构建 AIOps Agent 失败: {}", e)))?;

    // 构造初始 prompt（与 Go 版本一致的中文 prompt）
    let alert_prompt = r#"请分析当前系统的告警情况，执行以下步骤：
1. 首先查询 Prometheus 获取当前所有活动告警
2. 对每个告警，查询内部文档获取处理方法
3. 严格按照内部文档中的操作流程进行处理
4. 在查询时间相关数据前，先获取当前时间
5. 查询日志时，需要指定 region 和 topic_id 参数
6. 生成结构化的告警分析报告"#;

    // 运行 Plan-Execute-Replan 循环
    let report = run_aiops_loop(alert_prompt, &agents, 20).await?;

    tracing::info!(summary = %report.alert_summary, "AIOps 分析完成");

    Ok(Json(AioPsResponse {
        result: report.root_cause,
        detail: report.details,
    }))
}
