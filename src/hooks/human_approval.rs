use rig_core::agent::{PromptHook, ToolCallHookAction};
use rig_core::completion::CompletionModel;
use rig_core::wasm_compat::WasmCompatSend;
use std::collections::HashSet;
use std::future::Future;

/// 人工审批 Hook — 对指定工具的调用需要人工确认
///
/// 当 Agent 尝试调用需要审批的工具时，Hook 返回 `ToolCallHookAction::Skip`，
/// 将拒绝原因返回给 LLM，阻止工具执行。
///
/// 生产环境中可通过 channel 与前端交互，实现异步审批流程。
#[derive(Debug, Clone)]
pub struct HumanApprovalHook {
    /// 需要人工确认的工具名称集合
    approval_tools: HashSet<String>,
}

impl HumanApprovalHook {
    pub fn new(approval_tools: Vec<String>) -> Self {
        Self {
            approval_tools: approval_tools.into_iter().collect(),
        }
    }

    /// 创建默认的 Hook，MySQL CRUD 工具需要审批
    pub fn default_with_mysql() -> Self {
        Self::new(vec!["mysql_crud".to_string()])
    }
}

impl<M: CompletionModel> PromptHook<M> for HumanApprovalHook {
    fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> impl Future<Output = ToolCallHookAction> + WasmCompatSend {
        let needs_approval = self.approval_tools.contains(tool_name);
        let tool_name = tool_name.to_string();
        let args = args.to_string();
        async move {
            if !needs_approval {
                return ToolCallHookAction::cont();
            }
            tracing::warn!(
                tool = %tool_name,
                args = %args,
                "工具调用需要人工确认，已拦截"
            );
            ToolCallHookAction::skip(format!(
                "⚠️ 工具 `{}` 的操作需要人工确认。当前为自动模式，操作已被拦截。请通过审批接口确认后重试。",
                tool_name
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_approves_normal_tools() {
        let hook = HumanApprovalHook::default_with_mysql();
        assert!(hook.approval_tools.contains("mysql_crud"));
        assert!(!hook.approval_tools.contains("get_current_time"));
    }
}
