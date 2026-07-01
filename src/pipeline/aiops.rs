use rig_core::completion::Prompt;

use crate::agent::aiops_agent::AiopsAgents;
use crate::error::{AppError, AppResult};
use crate::pipeline::plan_types::*;

/// Plan-Execute-Replan 循环编排器
///
/// 流程:
/// 1. Planner (Think 模型) 从告警信息提取结构化计划
/// 2. Executor (Quick 模型 + Tools) 执行每个步骤
/// 3. Replanner (Think 模型) 评估结果，决定是否需要重新规划
/// 4. 循环直到完成或达到最大迭代次数
pub async fn run_aiops_loop(
    alert: &str,
    agents: &AiopsAgents,
    max_iterations: usize,
) -> AppResult<AioPsReport> {
    // Phase 1: 生成初始计划
    let mut plan = agents
        .plan_extractor
        .extract(alert)
        .await
        .map_err(|e| AppError::Internal(format!("计划提取失败: {}", e)))?;

    let mut details = Vec::new();
    let mut final_findings = String::new();

    tracing::info!(
        summary = %plan.alert_summary,
        severity = %plan.severity,
        steps = plan.steps.len(),
        "AIOps: 生成初始计划"
    );

    // Phase 2: 迭代执行
    for i in 0..max_iterations {
        tracing::info!(iteration = i + 1, "AIOps: 执行迭代");

        // 执行计划
        let prompt = format_plan_as_prompt(&plan);
        let result = agents
            .executor_agent
            .prompt(&prompt)
            .await
            .map_err(AppError::Prompt)?;

        details.push(format!("--- 迭代 {} ---\n{}", i + 1, result));
        final_findings = result;

        // 评估结果并决定是否 replan
        let investigation = agents
            .replan_extractor
            .extract(&final_findings)
            .await
            .map_err(|e| AppError::Internal(format!("重规划评估失败: {}", e)))?;

        if !investigation.needs_replan {
            tracing::info!(iteration = i + 1, "AIOps: 调查完成，无需重新规划");
            break;
        }

        if let Some(revised) = investigation.revised_plan {
            tracing::info!(
                iteration = i + 1,
                new_steps = revised.steps.len(),
                "AIOps: 重新规划"
            );
            plan = revised;
        } else {
            tracing::info!(iteration = i + 1, "AIOps: 无修订计划，结束循环");
            break;
        }

        if i == max_iterations - 1 {
            tracing::warn!(iterations = max_iterations, "AIOps: 达到最大迭代次数");
        }
    }

    // Phase 3: 生成最终报告
    let report = AioPsReport {
        alert_summary: plan.alert_summary.clone(),
        severity: plan.severity.clone(),
        root_cause: final_findings,
        details,
        recommendations: vec![], // 可通过额外的 Extractor 生成
    };

    Ok(report)
}
