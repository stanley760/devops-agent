use rig_core::completion::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;

/// MySQL CRUD 工具参数
#[derive(Deserialize, schemars::JsonSchema)]
pub struct MysqlCrudArgs {
    /// SQL 语句
    pub sql: String,
    /// 操作类型: select, insert, update, delete
    pub operation_type: String,
}

/// MySQL CRUD 工具错误
#[derive(Debug, thiserror::Error)]
pub enum MysqlError {
    #[error("数据库连接失败: {0}")]
    Pool(#[from] sqlx::Error),
    #[error("不支持的 SQL 操作类型: {0}")]
    UnsupportedOperation(String),
    #[error("SQL 执行失败: {0}")]
    Execution(String),
}

/// MySQL CRUD 工具 — 执行 SQL 语句，需要人工确认
#[derive(Debug, Clone)]
pub struct MysqlCrudTool {
    pool: sqlx::mysql::MySqlPool,
}

impl MysqlCrudTool {
    /// 从 DSN 创建 MySQL 连接池
    pub async fn new(dsn: &str) -> Result<Self, MysqlError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(dsn)
            .await?;
        Ok(Self { pool })
    }
}

impl Tool for MysqlCrudTool {
    const NAME: &'static str = "mysql_crud";
    type Error = MysqlError;
    type Args = MysqlCrudArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "执行 MySQL SQL 语句。支持 SELECT、INSERT、UPDATE、DELETE 操作。⚠️ 此工具需要人工确认后才会执行，特别是写操作（INSERT/UPDATE/DELETE）。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "要执行的 SQL 语句"
                    },
                    "operation_type": {
                        "type": "string",
                        "enum": ["select", "insert", "update", "delete"],
                        "description": "SQL 操作类型"
                    }
                },
                "required": ["sql", "operation_type"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let op = args.operation_type.to_lowercase();

        match op.as_str() {
            "select" => {
                let rows: Vec<serde_json::Value> = sqlx::query_scalar(&args.sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| MysqlError::Execution(e.to_string()))?;
                Ok(serde_json::to_string_pretty(&rows).unwrap_or_default())
            }
            "insert" | "update" | "delete" => {
                let result = sqlx::query(&args.sql)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| MysqlError::Execution(e.to_string()))?;

                let output = serde_json::json!({
                    "rows_affected": result.rows_affected(),
                    "operation": op,
                });
                Ok(serde_json::to_string_pretty(&output).unwrap_or_default())
            }
            _ => Err(MysqlError::UnsupportedOperation(op)),
        }
    }
}

// 注意：Serialize/Deserialize for MysqlCrudTool 需要特殊处理，
// 因为 MySqlPool 不实现 Serialize。
// 在 Agent 构建时通过 AppState 共享，不通过 serde 重建。
// 这里为 Tool trait 需要的 Clone 实现手动派生。
impl Serialize for MysqlCrudTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 序列化为占位符，实际重建通过 AppState
        serializer.serialize_str("mysql_crud_tool")
    }
}

impl<'de> Deserialize<'de> for MysqlCrudTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "MysqlCrudTool cannot be deserialized; use AppState to share the instance",
        ))
    }
}
