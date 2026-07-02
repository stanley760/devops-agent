// Re-export rig-core core types — 统一版本，避免下游 crate 版本冲突

pub use rig_core::completion::{
    CompletionError, CompletionModel, Prompt, PromptError, ToolDefinition,
};
pub use rig_core::agent::{Agent, AgentBuilder, MultiTurnStreamItem, PromptHook, ToolCallHookAction};
pub use rig_core::client::CompletionClient;
pub use rig_core::extractor::Extractor;
pub use rig_core::memory::{ConversationMemory, InMemoryConversationMemory, MemoryError};
pub use rig_core::message::Message;
pub use rig_core::streaming::{StreamedAssistantContent, StreamingPrompt};
pub use rig_core::tool::{Tool, ToolError, ToolSet};
pub use rig_core::vector_store::VectorStoreIndexDyn;
pub use rig_core::embeddings::EmbeddingsBuilder;
pub use rig_core::vector_store::InsertDocuments;
pub use rig_core::Embed;
