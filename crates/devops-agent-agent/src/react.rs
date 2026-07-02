//! React Agent 模式 — Agent + Tools + Memory + DynamicContext
//!
//! ReactAgent 是 rig Agent 的薄包装，提供：
//! - 惯用的 Builder API（ReactAgentBuilder）
//! - AgentRunner trait 实现（统一运行接口）
//! - Subagent trait 实现（可被其他 agent 委托）
//! - 直接访问流式输出（stream_prompt）

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use rig_core::agent::{Agent, AgentBuilder, PromptHook};
use rig_core::completion::{CompletionModel, Prompt, PromptError};

use devops_agent_core::error::SubagentError;
use devops_agent_core::traits::{AgentRunner, Subagent};

/// React Agent — rig Agent 的薄包装，实现 AgentRunner + Subagent
pub struct ReactAgent<M, P = ()>
where
    M: CompletionModel,
    P: PromptHook<M>,
{
    agent: Agent<M, P>,
    name: String,
    _phantom: PhantomData<P>,
}

impl<M, P> Clone for ReactAgent<M, P>
where
    M: CompletionModel + Clone,
    P: PromptHook<M> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            name: self.name.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<M, P> ReactAgent<M, P>
where
    M: CompletionModel,
    P: PromptHook<M>,
{
    /// 访问内部 rig Agent（用于 prompt、stream_prompt 等直接调用）
    pub fn inner(&self) -> &Agent<M, P> {
        &self.agent
    }

    /// Agent 名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<M, P> AgentRunner for ReactAgent<M, P>
where
    M: CompletionModel + Clone + 'static,
    P: PromptHook<M> + Clone + Send + Sync + 'static,
{
    type Input = String;
    type Output = String;
    type Error = PromptError;

    fn run(&self, input: Self::Input) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>> {
        let agent = self.agent.clone();
        Box::pin(async move { agent.prompt(&input).await })
    }
}

impl<M, P> Subagent for ReactAgent<M, P>
where
    M: CompletionModel + Clone + 'static,
    P: PromptHook<M> + Clone + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "React agent with tools, memory, and dynamic context"
    }

    fn delegate(&self, task: &str) -> Pin<Box<dyn Future<Output = Result<String, SubagentError>> + Send>> {
        let agent = self.agent.clone();
        let task = task.to_string();
        Box::pin(async move {
            agent
                .prompt(&task)
                .await
                .map_err(|e| SubagentError::Execution(e.to_string()))
        })
    }
}

/// 从 rig Agent 创建 ReactAgent 的便捷函数
impl<M, P> ReactAgent<M, P>
where
    M: CompletionModel,
    P: PromptHook<M>,
{
    /// 从已有的 rig Agent 创建 ReactAgent
    pub fn from_agent(agent: Agent<M, P>, name: &str) -> Self {
        Self {
            agent,
            name: name.to_string(),
            _phantom: PhantomData,
        }
    }
}

/// 便捷构建函数 — 从 AgentBuilder 构建 ReactAgent
///
/// # Example
/// ```no_run
/// use devops_agent_agent::react::ReactAgent;
/// use rig_core::agent::AgentBuilder;
///
/// let agent = AgentBuilder::new(model)
///     .name("my-agent")
///     .preamble("You are a helpful assistant")
///     .tool(my_tool)
///     .temperature(0.7)
///     .build();
/// let react_agent = ReactAgent::from_agent(agent, "my-agent");
/// ```
pub fn from_builder<M, P>(
    builder: AgentBuilder<M, P, rig_core::agent::WithBuilderTools>,
    name: &str,
) -> ReactAgent<M, P>
where
    M: CompletionModel,
    P: PromptHook<M>,
{
    ReactAgent::from_agent(builder.build(), name)
}
