use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use rig_core::vector_store::VectorStoreIndexDyn;
use rig_core::vector_store::request::VectorSearchRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 内部文档 RAG 查询工具参数
#[derive(Deserialize)]
pub struct DocRagArgs {
    /// 查询关键词
    pub query: String,
}

/// 内部文档 RAG 查询错误
#[derive(Debug, thiserror::Error)]
pub enum DocRagError {
    #[error("向量检索失败: {0}")]
    VectorStore(#[from] rig_core::vector_store::VectorStoreError),
    #[error("序列化失败: {0}")]
    Json(#[from] serde_json::Error),
}

/// 内部文档 RAG 查询工具 — 从 Milvus 向量库检索相关文档
#[derive(Clone)]
pub struct DocRagTool {
    index: Arc<dyn VectorStoreIndexDyn + Send + Sync>,
}

impl std::fmt::Debug for DocRagTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocRagTool").finish()
    }
}

impl DocRagTool {
    pub fn new(index: Arc<dyn VectorStoreIndexDyn + Send + Sync>) -> Self {
        Self { index }
    }
}

impl Tool for DocRagTool {
    const NAME: &'static str = "query_internal_docs";
    type Error = DocRagError;
    type Args = DocRagArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "查询内部文档知识库。从向量数据库中检索与查询相关的文档片段，包括告警处理手册、运维操作指南等。当需要查找特定告警的处理方法或内部操作流程时使用此工具。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "查询关键词或问题描述"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let request = VectorSearchRequest::builder()
            .query(&args.query)
            .samples(5)
            .build();
        let results = self.index.top_n(request).await?;

        let docs: Vec<serde_json::Value> = results
            .into_iter()
            .map(|(score, id, doc)| {
                serde_json::json!({
                    "id": id,
                    "score": score,
                    "document": doc,
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&docs).unwrap_or_default())
    }
}

// Serialize/Deserialize 需要特殊处理，因为 trait object 无法序列化
impl Serialize for DocRagTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("doc_rag_tool")
    }
}

impl<'de> Deserialize<'de> for DocRagTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "DocRagTool cannot be deserialized; use AppState to share the instance",
        ))
    }
}
