# devOpsAgent TODO

基于审计结果，以下是尚未实现或需要完善的功能清单。

## 🔴 P0 — 功能缺失（核心能力不可用）

### 1. MCP 工具未注册到 Agent
**文件**: `src/mcp/cls.rs`, `src/main.rs`
**问题**: `ClsMcpClient` 连接成功后，MCP 服务端发现的工具未注册到 `chat_agent` 或 `aiops_agent`。`ToolServerHandle` 创建后被丢弃，工具未流向任何 Agent。
**方案**: 将 `ToolServerHandle` 传入 AgentBuilder，使用 `.tool_server_handle(handle)` 共享 MCP 工具。需重构 Agent 构建流程以支持动态工具发现。
**Go 对应**: `internal/ai/tools/query_log.go` + `chat_pipeline/orchestration.go`

### 2. DocRagTool 未接入 AIOps Agent
**文件**: `src/agent/aiops_agent.rs`
**问题**: AIOps executor agent 无文档查询能力。系统 prompt 中声称有 `query_internal_docs` 工具，但 executor 的 tool list 中没有。
**方案**: 在 `build_aiops_agents` 中为 executor_agent 添加 `.tool(DocRagTool::new(index))` 或 `.dynamic_context(3, index)`。
**Go 对应**: `plan_execute_replan/executor.go` 中包含 `NewQueryInternalDocsTool`

### 3. MysqlCrudTool 未接入任何 Agent
**文件**: `src/tools/mysql.rs`, `src/agent/chat_agent.rs`
**问题**: `MYSQL_DSN` 配置存在但从未消费。MysqlCrudTool 实现完整但无 Agent 使用它。
**方案**: 在 `main.rs` 中根据 `mysql_dsn` 配置可选创建 MysqlCrudTool，传入 `build_chat_agent`。需要在 `AppState` 中存储可选的 MySQL 工具实例。
**Go 对应**: `chat_pipeline/flow.go` 中 `mysqlCrudTool` 绑定到 ReActAgent

## 🟡 P1 — 功能不完善（可用但需改进）

### 4. HumanApprovalHook 未注册到 Agent
**文件**: `src/hooks/human_approval.rs`
**问题**: Hook 实现完整但从未被任何 Agent 使用。MysqlCrudTool 接入后，需同时注册此 Hook。
**方案**: 当 MySQL 工具启用时，在 Agent 的 `.prompt()` 调用中传入 `.with_hook(HumanApprovalHook::default_with_mysql())`。或改为全局 hook 在 AgentBuilder 上设置。
**Go 对应**: `mysql_crud.go` 中的 stdin 人工确认

### 5. AIOps 报告中 recommendations 始终为空
**文件**: `src/pipeline/aiops.rs:86`
**问题**: `AioPsReport.recommendations` 始终为 `vec![]`，注释说"可通过额外的 Extractor 生成"。
**方案**: 在循环结束后，使用 Extractor 从 final_findings 中提取建议列表。
**Go 对应**: Go 版本的 plan-execute-replan 最终也会生成结构化建议

### 6. SSE 工具函数未被使用
**文件**: `src/sse.rs`
**问题**: `rig_stream_to_sse()` 是独立工具函数，但 `chat_stream.rs` 内联了自己的转换逻辑，造成重复。
**方案**: 重构 `chat_stream.rs` 使用 `sse::rig_stream_to_sse()`，或将 `sse.rs` 中的函数删除，保留内联版本。

### 7. 响应格式缺少统一包装
**文件**: `src/api/*.rs`
**问题**: Go 版本有 `ResponseMiddleware` 将所有响应包装为 `{"message": "OK", "data": ...}` 格式。Rust 版本返回裸 JSON。
**方案**: 添加 axum middleware 或在 error.rs 的 `IntoResponse` 中统一包装格式。
**Go 对应**: `utility/middleware/middleware.go` 的 `ResponseMiddleware`

## 🟢 P2 — 体验优化（非核心但有价值）

### 8. Markdown 文档分块器
**文件**: `src/api/upload.rs`
**问题**: 当前使用 `split("\n\n")` 按段落分块。Go 版本使用 `markdown.HeaderSplitter` 按 `#` 标题层级分块，更精确。
**方案**: 实现或引入 Markdown 解析 crate（如 `pulldown-cmark`），按标题层级分块。
**Go 对应**: `knowledge_index_pipeline/orchestration.go` 中 `markdown.NewHeaderSplitter()`

### 9. 文档去重（重新上传时删除旧文档）
**文件**: `src/api/upload.rs`
**问题**: 重复上传同名文件会创建重复条目。Go 版本在索引前查询并删除同 `_source` 的旧文档。
**方案**: 在 `index_document` 中先查询 `Filter::eq("_source", source)`，删除匹配文档后再插入。
**Go 对应**: `knowledge_cmd/main.go` 和 `chat_v1_file_upload.go` 中的删除逻辑

### 10. Milvus 自动建库建表
**文件**: `src/vector_store/milvus_store.rs`
**问题**: 依赖 Milvus 中已存在 `agent` 数据库和 `biz` 集合。Go 版本启动时自动创建。
**方案**: 在 `create_milvus_store` 中添加 Milvus REST API 调用：创建数据库 → 创建集合 → 创建索引（如果不存在）。
**Go 对应**: `utility/client/client.go` 的 `NewMilvusClient()` 自动创建

### 11. 健康检查端点
**文件**: `src/api/mod.rs`
**问题**: 无健康检查 API。
**方案**: 添加 `GET /api/health` 返回各组件状态（Milvus、LLM、Prometheus 连通性）。

### 12. CLI 子命令（独立测试工具）
**问题**: Go 版本有 5 个 CLI 命令独立测试各组件。Rust 版本只有 HTTP 服务器。
**方案**: 使用 `clap` 添加子命令：
- `devOpsAgent serve` — 启动 HTTP 服务器（默认）
- `devOpsAgent chat` — 交互式对话测试
- `devOpsAgent aiops` — AIOps 流水线测试
- `devOpsAgent index <dir>` — 批量索引文档
- `devOpsAgent recall <query>` — RAG 检索测试
**Go 对应**: `internal/ai/cmd/` 下的 5 个命令

### 13. 结构化日志与 PromptHook 回调
**问题**: Go 版本有 eino Callback 系统记录每个组件的起止。Rust 版本仅用 tracing。
**方案**: 实现 `PromptHook` 的 `on_completion_call` / `on_tool_call` / `on_tool_result` 方法，记录 LLM 调用和工具执行的详细信息（耗时、token 用量等）。
**Go 对应**: `utility/log_call_back/log_call_back.go`

## 🔵 P3 — 部署与文档

### 14. Dockerfile
**方案**: 多阶段构建 `rust:1.85-alpine` → `gcr.io/distroless/cc`，最终镜像 < 50MB。

### 15. docker-compose.yml
**方案**: 包含 Milvus (etcd + minio + milvus-standalone) + Attu 管理界面 + devOpsAgent。

### 16. 示例文档
**方案**: 添加 `docs/告警处理手册.md` 示例文件，包含常见告警（服务下线、高错误率、对账不一致等）的处理步骤。

### 17. 集成测试
**方案**: 添加 `tests/` 目录：
- API 端点测试（使用 `axum::test` + `tower::ServiceExt`）
- Agent 行为测试（mock LLM 响应）
- 工具集成测试（mock Prometheus/MySQL/Milvus）

### 18. 前端
**方案**: 移植 `SuperBizAgentFrontend/` 或开发新的前端界面。Go 版本前端功能：Quick/Stream 双模式、AIOps 按钮、文件上传、聊天历史侧边栏、Markdown 渲染。
