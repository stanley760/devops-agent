use std::future::Future;
use std::pin::Pin;

use serde::de::DeserializeOwned;
use serde::Serialize;
use schemars::JsonSchema;

use crate::error::SubagentError;

/// 统一"运行 agent"接口
///
/// 所有 agent 类型（React、Plan-Execute、Supervisor 等）都实现此 trait，
/// 提供统一的 `run()` 方法。这使得 agent 可以互换使用，
/// 也使得 Pipeline 可以组合任意 AgentRunner。
pub trait AgentRunner: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;
    type Error: std::error::Error + Send + Sync;

    fn run(&self, input: Self::Input) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;
}

/// 可被其他 agent 委托的子 agent
///
/// Subagent 是多 agent 协作的核心抽象：
/// - 每个 Subagent 有 name 和 description，可用于路由决策
/// - `delegate()` 方法让父 agent 将任务委托给子 agent
/// - 通过 `SubagentTool` 可将 Subagent 暴露为 rig Tool，让 LLM 直接调用
///
/// 使用 `Pin<Box<dyn Future>>` 返回类型以确保 dyn-compatible（Supervisor 中用 `Box<dyn Subagent>`）
pub trait Subagent: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    fn delegate(&self, task: &str) -> Pin<Box<dyn Future<Output = Result<String, SubagentError>> + Send>>;
}

/// 通用计划 trait
///
/// Plan-Execute-Replan 模式的核心抽象。业务类型（如 AioPsPlan）实现此 trait
/// 即可使用 `PlanExecuteAgents` 和 `run_plan_execute_loop`。
pub trait Plan:
    Serialize + DeserializeOwned + JsonSchema + Clone + Send + Sync + 'static
{
    type Step: Step;

    /// 计划中的步骤列表
    fn steps(&self) -> &[Self::Step];

    /// 计划摘要
    fn summary(&self) -> &str;

    /// 将计划格式化为 executor agent 可理解的 prompt
    ///
    /// 默认实现按步骤格式化，业务类型可覆盖以自定义格式
    fn format_as_prompt(&self) -> String {
        let steps: Vec<String> = self
            .steps()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "步骤 {}: 使用工具 `{}` — 目的: {}",
                    i + 1,
                    s.tool(),
                    s.purpose()
                )
            })
            .collect();

        format!(
            "请按以下计划执行调查：\n\n摘要: {}\n\n步骤:\n{}\n\n请逐步执行以上步骤，每步调用相应工具并报告结果。",
            self.summary(),
            steps.join("\n")
        )
    }
}

/// 通用步骤 trait
pub trait Step:
    Serialize + DeserializeOwned + JsonSchema + Clone + Send + Sync + 'static
{
    /// 步骤使用的工具名称
    fn tool(&self) -> &str;

    /// 步骤目的
    fn purpose(&self) -> &str;
}
