//! AIOps 业务计划类型 — 实现 devops-agent-core 的 Plan/Step trait
//!
//! 这些类型是 AIOps 领域特定的，保留在主 crate 中。
//! 通用计划逻辑（run_plan_execute_loop 等）在 devops-agent-agent crate 中。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use devops_agent_core::traits::{Plan, Step};

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

impl Plan for AioPsPlan {
    type Step = InvestigationStep;

    fn steps(&self) -> &[Self::Step] {
        &self.steps
    }

    fn summary(&self) -> &str {
        &self.alert_summary
    }

    /// AIOps 自定义格式：包含告警摘要、严重程度和期望发现
    fn format_as_prompt(&self) -> String {
        let steps: Vec<String> = self
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
            self.alert_summary,
            self.severity,
            steps.join("\n")
        )
    }
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

impl Step for InvestigationStep {
    fn tool(&self) -> &str {
        &self.tool
    }

    fn purpose(&self) -> &str {
        &self.purpose
    }
}
