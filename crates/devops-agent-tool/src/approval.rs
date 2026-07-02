//! 通用审批 Hook
//!
//! 泛化自 HumanApprovalHook，对指定工具的调用需要审批确认。
//! 默认行为是自动拦截需要审批的工具调用（Skip），
//! 可通过 `with_callback` 自定义审批逻辑（如与前端交互）。

use std::collections::HashSet;
use std::future::Future;

use rig_core::agent::{PromptHook, ToolCallHookAction};
use rig_core::completion::CompletionModel;
use rig_core::wasm_compat::WasmCompatSend;

/// 通用审批 Hook — 对指定工具的调用需要审批确认
///
/// 当 Agent 尝试调用需要审批的工具时：
/// - 默认行为：返回 `ToolCallHookAction::Skip`，将拒绝原因返回给 LLM
/// - 自定义行为：通过 `with_callback` 设置审批回调，返回 true 放行，false 拦截
///
/// # Example
/// ```no_run
/// use devops_agent_tool::approval::ApprovalHook;
///
/// // 默认：自动拦截
/// let hook = ApprovalHook::new(vec!["mysql_crud".to_string()]);
///
/// // 自定义：与前端交互
/// let hook = ApprovalHook::new(vec!["mysql_crud".to_string()])
///     .with_callback(|tool, args| {
///         println!("工具 {} 需要审批，参数: {}", tool, args);
///         true  // 审批通过
///     });
/// ```
#[derive(Clone)]
pub struct ApprovalHook {
    /// 需要审批的工具名称集合
    approval_tools: HashSet<String>,
    /// 审批回调：(tool_name, args) → approved?
    on_approval: fn(&str, &str) -> bool,
}

impl std::fmt::Debug for ApprovalHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalHook")
            .field("approval_tools", &self.approval_tools)
            .finish()
    }
}

impl ApprovalHook {
    /// 创建审批 Hook，指定需要审批的工具列表
    ///
    /// 默认行为：所有审批工具的调用都被自动拦截
    pub fn new(approval_tools: Vec<String>) -> Self {
        Self {
            approval_tools: approval_tools.into_iter().collect(),
            on_approval: |_tool, _args| false, // 默认拒绝
        }
    }

    /// 创建默认的 Hook，MySQL CRUD 工具需要审批
    pub fn default_with_mysql() -> Self {
        Self::new(vec!["mysql_crud".to_string()])
    }

    /// 设置审批回调
    ///
    /// 回调返回 true 表示审批通过（放行），false 表示拒绝（拦截）
    pub fn with_callback(mut self, f: fn(&str, &str) -> bool) -> Self {
        self.on_approval = f;
        self
    }

    /// 获取需要审批的工具名称集合
    pub fn approval_tools(&self) -> &HashSet<String> {
        &self.approval_tools
    }
}

impl<M: CompletionModel> PromptHook<M> for ApprovalHook {
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
        let on_approval = self.on_approval;
        async move {
            if !needs_approval {
                return ToolCallHookAction::cont();
            }

            let approved = on_approval(&tool_name, &args);
            if approved {
                tracing::info!(
                    tool = %tool_name,
                    "工具调用审批通过"
                );
                ToolCallHookAction::cont()
            } else {
                tracing::warn!(
                    tool = %tool_name,
                    args = %args,
                    "工具调用需要审批，已拦截"
                );
                ToolCallHookAction::skip(format!(
                    "⚠️ 工具 `{}` 的操作需要人工确认。当前为自动模式，操作已被拦截。请通过审批接口确认后重试。",
                    tool_name
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_approves_normal_tools() {
        let hook = ApprovalHook::default_with_mysql();
        assert!(hook.approval_tools.contains("mysql_crud"));
        assert!(!hook.approval_tools.contains("get_current_time"));
    }

    #[test]
    fn test_hook_with_callback_approve() {
        let hook = ApprovalHook::new(vec!["test_tool".to_string()])
            .with_callback(|_tool, _args| true);
        // The callback returns true, so tools should be approved
        assert!((hook.on_approval)("test_tool", "args"));
    }

    #[test]
    fn test_hook_with_callback_deny() {
        let hook = ApprovalHook::new(vec!["test_tool".to_string()]);
        // Default callback returns false
        assert!(!(hook.on_approval)("test_tool", "args"));
    }
}
