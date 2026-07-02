//! 人工审批 Hook
//!
//! 使用 devops-agent-tool 框架 crate 的 ApprovalHook 通用审批 hook。
//! 业务特定逻辑（知道 "mysql_crud" 需要审批）保留在此。

pub use devops_agent_tool::approval::ApprovalHook as HumanApprovalHook;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_approves_normal_tools() {
        let hook = HumanApprovalHook::default_with_mysql();
        assert!(hook.approval_tools().contains("mysql_crud"));
        assert!(!hook.approval_tools().contains("get_current_time"));
    }
}
