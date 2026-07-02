//! Supervisor — 多 agent 调度器
//!
//! 根据输入分类，将任务分发给最合适的子 agent。
//!
//! 注意：Supervisor 不实现 AgentRunner/Subagent trait，
//! 因为 `Box<dyn Subagent>` 的 delegate() 返回 `'static` future，
//! 但访问 `self.agents[idx]` 借用了 `self`，无法满足 `'static` 要求。
//! 业务代码应直接调用 `dispatch()` 方法。


use devops_agent_core::error::SubagentError;
use devops_agent_core::traits::Subagent;

/// Supervisor — 多 agent 调度器
///
/// 根据分类函数将任务分发给注册的子 agent。
///
/// # Example
/// ```no_run
/// use devops_agent_agent::supervisor::Supervisor;
///
/// let supervisor = Supervisor::new()
///     .add_agent(Box::new(chat_agent))
///     .add_agent(Box::new(aiops_agent))
///     .classifier(|input| {
///         if input.contains("告警") { Some(1) }  // aiops
///         else { Some(0) }                        // chat
///     });
///
/// let result = supervisor.dispatch("查询告警").await?;
/// ```
pub struct Supervisor {
    agents: Vec<Box<dyn Subagent>>,
    classifier: Box<dyn Fn(&str) -> Option<usize> + Send + Sync>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            classifier: Box::new(|_| None),
        }
    }

    /// 添加子 agent
    pub fn add_agent(mut self, agent: Box<dyn Subagent>) -> Self {
        self.agents.push(agent);
        self
    }

    /// 设置分类函数：输入 → 子 agent 索引（None 表示无法分类）
    pub fn classifier(mut self, f: impl Fn(&str) -> Option<usize> + Send + Sync + 'static) -> Self {
        self.classifier = Box::new(f);
        self
    }

    /// 分发任务给合适的子 agent
    pub async fn dispatch(&self, task: &str) -> Result<String, SubagentError> {
        let idx = (self.classifier)(task).ok_or_else(|| {
            SubagentError::Refused(format!("无法分类任务: {}", task))
        })?;

        let agent = self.agents.get(idx).ok_or_else(|| {
            SubagentError::Refused(format!("子 agent 索引越界: {}", idx))
        })?;

        agent.delegate(task).await
    }

    /// 列出所有注册的子 agent 名称
    pub fn list_agents(&self) -> Vec<&str> {
        self.agents.iter().map(|a| a.name()).collect()
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}
