//! Plan-Execute-Replan 三阶段循环模式
//!
//! 通用化提取自 AIOps pipeline，支持任意实现 Plan/Step trait 的计划类型。

use std::marker::PhantomData;

use rig_core::agent::Agent;
use rig_core::completion::{CompletionModel, Prompt};
use rig_core::extractor::Extractor;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use devops_agent_core::error::PlanExecuteError;
use devops_agent_core::traits::{Plan, Step};

// ─── 默认计划类型 ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DefaultPlan {
    pub summary: String,
    pub severity: String,
    pub steps: Vec<DefaultStep>,
}

impl Plan for DefaultPlan {
    type Step = DefaultStep;
    fn steps(&self) -> &[Self::Step] { &self.steps }
    fn summary(&self) -> &str { &self.summary }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DefaultStep {
    pub tool: String,
    pub purpose: String,
    pub expected_findings: String,
}

impl Step for DefaultStep {
    fn tool(&self) -> &str { &self.tool }
    fn purpose(&self) -> &str { &self.purpose }
}

// ─── 重规划与报告 ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(bound = "P: Plan")]
pub struct ReplanResult<P: Plan> {
    pub step_completed: String,
    pub findings: String,
    pub needs_replan: bool,
    pub revised_plan: Option<P>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "P: Plan")]
pub struct PlanExecuteReport<P: Plan> {
    pub summary: String,
    pub root_cause: String,
    pub details: Vec<String>,
    pub recommendations: Vec<String>,
    _phantom: PhantomData<P>,
}

// ─── 三阶段 Agent 集合 ───

/// Plan-Execute-Replan 三阶段 Agent 集合
pub struct PlanExecuteAgents<M: CompletionModel + 'static, P: Plan> {
    pub planner: Extractor<M, P>,
    pub executor: Agent<M>,
    pub replanner: Extractor<M, ReplanResult<P>>,
}

// ─── Builder ───

pub struct PlanExecuteAgentsBuilder<M, P: Plan> {
    _phantom: PhantomData<(M, P)>,
}

impl<M: CompletionModel + Clone + 'static, P: Plan> PlanExecuteAgentsBuilder<M, P> {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }

    /// 构建 PlanExecuteAgents
    pub fn build(
        self,
        planner: Extractor<M, P>,
        executor: Agent<M>,
        replanner: Extractor<M, ReplanResult<P>>,
    ) -> PlanExecuteAgents<M, P> {
        PlanExecuteAgents { planner, executor, replanner }
    }
}

impl<M: CompletionModel + Clone + 'static, P: Plan> Default for PlanExecuteAgentsBuilder<M, P> {
    fn default() -> Self { Self::new() }
}

// ─── 编排循环 ───

/// 运行 Plan-Execute-Replan 循环
pub async fn run_plan_execute_loop<M, P: Plan>(
    input: &str,
    agents: &PlanExecuteAgents<M, P>,
    max_iterations: usize,
) -> Result<PlanExecuteReport<P>, PlanExecuteError>
where
    M: CompletionModel + 'static,
{
    let mut plan = agents
        .planner
        .extract(input)
        .await
        .map_err(|e| PlanExecuteError::PlanExtraction(e.to_string()))?;

    let mut details = Vec::new();
    let mut final_findings = String::new();

    tracing::info!(summary = %plan.summary(), steps = plan.steps().len(), "Plan-Execute: 生成初始计划");

    for i in 0..max_iterations {
        tracing::info!(iteration = i + 1, "Plan-Execute: 执行迭代");

        let prompt = plan.format_as_prompt();
        let result = agents.executor.prompt(&prompt).await?;

        details.push(format!("--- 迭代 {} ---\n{}", i + 1, result));
        final_findings = result;

        let investigation = agents
            .replanner
            .extract(&final_findings)
            .await
            .map_err(|e| PlanExecuteError::ReplanEvaluation(e.to_string()))?;

        if !investigation.needs_replan {
            tracing::info!(iteration = i + 1, "Plan-Execute: 调查完成，无需重新规划");
            break;
        }

        if let Some(revised) = investigation.revised_plan {
            tracing::info!(iteration = i + 1, new_steps = revised.steps().len(), "Plan-Execute: 重新规划");
            plan = revised;
        } else {
            tracing::info!(iteration = i + 1, "Plan-Execute: 无修订计划，结束循环");
            break;
        }

        if i == max_iterations - 1 {
            tracing::warn!(iterations = max_iterations, "Plan-Execute: 达到最大迭代次数");
        }
    }

    Ok(PlanExecuteReport {
        summary: plan.summary().to_string(),
        root_cause: final_findings,
        details,
        recommendations: vec![],
        _phantom: PhantomData,
    })
}

// Note: PlanExecuteAgents intentionally does NOT implement AgentRunner/Subagent
// because Extractor<M, T> does not implement Clone, making it impossible to
// move `&self` into a `'static` future. Business code should call
// `run_plan_execute_loop()` directly or wrap it in an Arc if needed.
