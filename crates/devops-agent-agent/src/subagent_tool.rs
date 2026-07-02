//! SubagentTool — 将 Subagent 暴露为 rig Tool
//!
//! 让父 agent 可通过工具调用委托子 agent，实现多 agent 协作。
//! LLM 会看到子 agent 的 name/description 作为工具定义，
//! 调用时自动委托给子 agent 执行。

use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::Deserialize;

use devops_agent_core::error::SubagentError;
use devops_agent_core::traits::Subagent;

/// SubagentTool 参数
#[derive(Debug, Deserialize)]
pub struct SubagentToolArgs {
    /// 委托给子 agent 的任务描述
    pub task: String,
}

/// 将 Subagent 暴露为 rig Tool
///
/// # Example
/// ```no_run
/// use devops_agent_agent::subagent_tool::SubagentTool;
///
/// // 将子 agent 包装为工具，注册到父 agent
/// let subagent_tool = SubagentTool::new(my_subagent);
/// let parent_agent = AgentBuilder::new(model)
///     .tool(subagent_tool)
///     .build();
/// ```
pub struct SubagentTool<S: Subagent> {
    subagent: S,
}

impl<S: Subagent> SubagentTool<S> {
    pub fn new(subagent: S) -> Self {
        Self { subagent }
    }
}

impl<S: Subagent + 'static> Tool for SubagentTool<S> {
    const NAME: &'static str = "delegate_to_subagent";
    type Error = SubagentError;
    type Args = SubagentToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: self.subagent.name().to_string(),
            description: format!(
                "委托任务给专门的子 agent「{}」。{}",
                self.subagent.name(),
                self.subagent.description()
            ),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": format!("委托给子 agent「{}」的任务描述", self.subagent.name())
                    }
                },
                "required": ["task"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        tracing::info!(
            subagent = %self.subagent.name(),
            task = %args.task,
            "SubagentTool: 委托任务"
        );
        self.subagent.delegate(&args.task).await
    }
}
