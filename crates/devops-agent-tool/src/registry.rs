//! 动态工具注册表
//!
//! 提供工具的注册、查找和列举功能，
//! 为未来从 MCP 服务端自动发现工具预留扩展点。

use std::collections::HashMap;

use rig_core::completion::ToolDefinition;

/// 工具注册条目
#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub definition: ToolDefinition,
}

/// 动态工具注册表
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolEntry>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 直接注册一个工具定义
    pub fn register_definition(&mut self, definition: ToolDefinition) {
        self.tools.insert(
            definition.name.clone(),
            ToolEntry {
                name: definition.name.clone(),
                definition,
            },
        );
    }

    /// 列出所有已注册工具的名称
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// 查找工具定义
    pub fn get(&self, name: &str) -> Option<&ToolEntry> {
        self.tools.get(name)
    }

    /// 已注册工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
