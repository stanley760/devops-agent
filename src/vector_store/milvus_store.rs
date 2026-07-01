use rig_core::embeddings::EmbeddingModel;
use rig_milvus::MilvusVectorStore;

/// 创建 MilvusVectorStore — 同时实现了 VectorStoreIndex + VectorStoreIndexDyn + InsertDocuments
///
/// 可直接传递给 AgentBuilder::dynamic_context()，也可用于 insert_documents()
pub fn create_milvus_store<M: EmbeddingModel + Send + Sync + Clone + 'static>(
    embedding_model: M,
    base_url: &str,
    database: &str,
    collection: &str,
    username: Option<String>,
    password: Option<String>,
) -> MilvusVectorStore<M> {
    let mut store = MilvusVectorStore::new(
        embedding_model,
        base_url.to_string(),
        database.to_string(),
        collection.to_string(),
    );

    if let (Some(user), Some(pass)) = (username, password) {
        store = store.auth(user, pass);
    }

    store
}
