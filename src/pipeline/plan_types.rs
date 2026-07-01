use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AIOps 计划 — 从告警信息生成的调查计划
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AioPsPlan {
    /// 告警摘要
    pub alert_summary: String,
    /// 严重程度 (critical/warning/info)
    pub severity: String,
    /// 调查步骤列表
    pub steps: Vec<InvestigationStep>,
}

/// 调查步骤 — 计划中的单个步骤
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InvestigationStep {
    /// 要使用的工具名称
    pub tool: String,
    /// 步骤目的
    pub purpose: String,
    /// 期望的发现
    pub expected_findings: String,
}

/// 调查结果 — 执行步骤后的结果和判断
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InvestigationResult {
    /// 已完成的步骤描述
    pub step_completed: String,
    /// 调查发现
    pub findings: String,
    /// 是否需要重新规划
    pub needs_replan: bool,
    /// 修订后的计划（如果需要重新规划）
    pub revised_plan: Option<AioPsPlan>,
}

/// AIOps 最终报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AioPsReport {
    /// 告警摘要
    pub alert_summary: String,
    /// 严重程度
    pub severity: String,
    /// 根因分析
    pub root_cause: String,
    /// 调查详情（每个迭代的记录）
    pub details: Vec<String>,
    /// 建议措施
    pub recommendations: Vec<String>,
}

/// 将计划格式化为 Agent 可执行的 prompt
pub fn format_plan_as_prompt(plan: &AioPsPlan) -> String {
    let steps: Vec<String> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            format!(
                "步骤 {}: 使用工具 `{}` — 目的: {} — 期望发现: {}",
                i + 1,
                step.tool,
                step.purpose,
                step.expected_findings
            )
        })
        .collect();

    format!(
        "请按以下计划执行调查：\n\n\
         告警摘要: {}\n\
         严重程度: {}\n\n\
         调查步骤:\n{}\n\n\
         请逐步执行以上步骤，每步调用相应工具并报告结果。",
        plan.alert_summary,
        plan.severity,
        steps.join("\n")
    )
}
