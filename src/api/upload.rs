use axum::extract::{Multipart, State};
use axum::Json;
use rig_core::embeddings::EmbeddingsBuilder;
use rig_core::vector_store::InsertDocuments;
use rig_core::Embed;
use serde::Serialize;
use std::path::Path;

use crate::error::AppError;
use crate::AppState;

/// 文件上传响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
}

/// 文档结构体，实现 Embed trait 用于向量嵌入
#[derive(Debug, serde::Serialize, rig_core::Embed)]
struct Doc {
    content: String,
    #[embed]
    embedded_text: String,
    _source: String,
}

/// POST /api/upload — 文件上传并索引到向量库
///
/// 接收 .txt 或 .md 文件，将内容分块嵌入后存入 Milvus 向量库
pub async fn upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let mut file_name = String::new();
    let mut file_path = String::new();
    let mut file_size: u64 = 0;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Internal(format!("读取上传字段失败: {}", e))
    })? {
        let _name = field.name().unwrap_or("unknown").to_string();
        let filename = field
            .file_name()
            .unwrap_or("unnamed")
            .to_string();

        let data = field.bytes().await.map_err(|e| {
            AppError::Internal(format!("读取文件数据失败: {}", e))
        })?;

        file_size = data.len() as u64;
        file_name = filename.clone();

        // 确保上传目录存在
        let upload_dir = Path::new(&state.config.file_dir);
        tokio::fs::create_dir_all(upload_dir).await?;

        // 保存文件
        let save_path = upload_dir.join(&filename);
        file_path = save_path.to_string_lossy().to_string();
        tokio::fs::write(&save_path, &data).await?;

        tracing::info!(file = %filename, size = file_size, "文件已保存");

        // 将文档内容索引到 Milvus 向量库
        if let Ok(content) = String::from_utf8(data.to_vec()) {
            index_document(&state, &content, &filename).await?;
        }
    }

    Ok(Json(UploadResponse {
        file_name,
        file_path,
        file_size,
    }))
}

/// 将文档内容分块并索引到 Milvus
async fn index_document(
    state: &AppState,
    content: &str,
    source: &str,
) -> Result<(), AppError> {
    // 简单分块：按段落分割
    let chunks: Vec<&str> = content
        .split("\n\n")
        .filter(|chunk| !chunk.trim().is_empty())
        .collect();

    if chunks.is_empty() {
        return Ok(());
    }

    // 为每个 chunk 创建文档
    let documents: Vec<Doc> = chunks
        .into_iter()
        .map(|chunk| Doc {
            content: chunk.to_string(),
            embedded_text: chunk.to_string(),
            _source: source.to_string(),
        })
        .collect();

    // 嵌入并存储
    let embeddings = EmbeddingsBuilder::new(state.embedding_model.clone())
        .documents(documents)
        .map_err(|e| AppError::Internal(format!("构建嵌入失败: {}", e)))?
        .build()
        .await
        .map_err(|e| AppError::Internal(format!("生成嵌入失败: {}", e)))?;

    state
        .milvus_store
        .insert_documents(embeddings)
        .await
        .map_err(AppError::VectorStore)?;

    tracing::info!(source = %source, "文档索引完成");
    Ok(())
}
