use rig_core::completion::PromptError;

/// Subagent 委托错误
#[derive(Debug, thiserror::Error)]
pub enum SubagentError {
    #[error("Agent 执行失败: {0}")]
    Execution(String),

    #[error("Agent 委托超时")]
    Timeout,

    #[error("Agent 拒绝任务: {0}")]
    Refused(String),
}

impl From<PromptError> for SubagentError {
    fn from(e: PromptError) -> Self {
        SubagentError::Execution(e.to_string())
    }
}

/// Plan-Execute 循环错误
#[derive(Debug, thiserror::Error)]
pub enum PlanExecuteError {
    #[error("计划提取失败: {0}")]
    PlanExtraction(String),

    #[error("执行失败: {0}")]
    Execution(#[from] PromptError),

    #[error("重规划评估失败: {0}")]
    ReplanEvaluation(String),
}
