use rig_core::agent::Agent;
use rig_core::completion::CompletionModel;
use rig_core::memory::InMemoryConversationMemory;
use rig_core::message::Message;
use rig_core::vector_store::VectorStoreIndexDyn;

use crate::tools::current_time::CurrentTimeTool;
use crate::tools::prometheus::PrometheusAlertsTool;

/// Chat Agent 系统 prompt
const CHAT_SYSTEM_PROMPT: &str = r#"你是一个智能 OnCall/Ops 助手，专注于帮助 SRE 和运维团队处理告警、排查问题。

你的核心能力:
1. 查询 Prometheus 告警 — 了解当前系统告警状态
2. 查询内部文档 — 从知识库检索告警处理手册和运维指南（通过上下文自动检索）
3. 搜索日志 — 通过 MCP 查询腾讯云 CLS 日志
4. 执行数据库操作 — MySQL CRUD（需要人工确认）
5. 获取当前时间 — 用于时间相关查询

工作原则:
- 在查询时间相关数据前，先调用 get_current_time 获取当前时间
- 严格按照内部文档中的操作流程处理告警
- 查询日志时，需要指定 region 和 topic_id 参数
- 数据库写操作需要人工确认
- 如果不确定，先查询文档再操作"#;

/// 构建 Chat Agent
///
/// 包含 RAG 上下文检索（dynamic_context）、对话记忆（6条滑动窗口）、工具
/// 注意: DocRagTool 已通过 dynamic_context 自动提供 RAG 检索，
/// 不需要额外的工具来查询向量库。
pub fn build_chat_agent<M, I>(
    model: M,
    index: I,
    prometheus_url: String,
) -> Agent<M>
where
    M: CompletionModel + Clone + 'static,
    I: VectorStoreIndexDyn + Send + Sync + 'static,
{
    // 滑动窗口记忆，保留最近 6 条消息
    let memory = InMemoryConversationMemory::new().with_filter(|msgs: Vec<Message>| {
        let len = msgs.len();
        if len <= 6 {
            msgs
        } else {
            msgs.into_iter().skip(len - 6).collect()
        }
    });

    rig_core::agent::AgentBuilder::new(model)
        .name("chat-agent")
        .preamble(CHAT_SYSTEM_PROMPT)
        .dynamic_context(3, index)
        .tool(PrometheusAlertsTool::new(prometheus_url))
        .tool(CurrentTimeTool)
        .memory(memory)
        .temperature(0.7)
        .max_tokens(4096)
        .default_max_turns(5)
        .build()
}
