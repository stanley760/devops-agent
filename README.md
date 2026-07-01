# devOpsAgent

基于 Rust + [rig-core](https://github.com/0xPlaygrounds/rig) 的智能 OnCall/Ops 助手，重写自 Go 语言 [SuperBizAgent](https://github.com/0xPlaygrounds/rig) 项目。

通过 RAG 增强对话、Plan-Execute-Replan AIOps 告警分析、多工具协同，自动化处理运维告警和故障排查。

## 功能概览

| 功能 | 说明 |
|------|------|
| 🔍 RAG 增强对话 | 基于 Milvus 向量库自动检索相关文档，增强 LLM 上下文 |
| 🤖 AIOps 告警分析 | Plan-Execute-Replan 三阶段循环，自动查询告警 → 制定计划 → 执行调查 → 评估重规划 |
| 📊 Prometheus 告警查询 | 查询活动告警，自动去重和计算持续时间 |
| 📝 内部文档检索 | 从知识库检索告警处理手册、运维指南 |
| 🔎 日志搜索 (MCP) | 通过 rmcp 连接腾讯云 CLS MCP 服务端查询日志 |
| 🗄️ MySQL CRUD | SQL 执行，写操作支持人工审批拦截 |
| ⏰ 当前时间 | 多格式时间返回 |
| 📤 文件上传 + 知识索引 | 上传文档 → 分块嵌入 → 存入 Milvus |
| 💬 SSE 流式对话 | Server-Sent Events 流式输出，支持文本/工具调用/推理过程 |
| 🧠 对话记忆 | 会话级滑动窗口记忆（6 条消息） |

## 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust 2024 Edition |
| AI 框架 | rig-core 0.38.2 + rig-milvus 0.38.2 |
| LLM | DeepSeek V3 via Volcengine (OpenAI 兼容) |
| Embedding | DashScope text-embedding-v4 (OpenAI 兼容) |
| 向量库 | Milvus v2.5+ |
| Web 框架 | axum 0.8 + tower-http (CORS) |
| 异步运行时 | tokio |
| MCP 客户端 | rmcp 1.7 |
| 数据库 | sqlx (MySQL) |
| 日志 | tracing + tracing-subscriber |

## 项目结构

```
src/
├── main.rs                    # 入口：配置加载、客户端构建、服务器启动
├── config.rs                  # 类型化配置（环境变量）
├── error.rs                   # 统一错误类型 + axum IntoResponse
├── sse.rs                     # rig Stream → axum SSE 转换工具
│
├── agent/                     # Agent 构建
│   ├── chat_agent.rs          #   RAG 对话 Agent
│   └── aiops_agent.rs         #   AIOps Plan/Execute/Replan Agent
│
├── tools/                     # 工具定义 (rig Tool trait)
│   ├── prometheus.rs          #   Prometheus 告警查询
│   ├── current_time.rs        #   当前时间
│   ├── mysql.rs               #   MySQL CRUD
│   └── doc_rag.rs             #   内部文档 RAG 查询
│
├── hooks/                     # Agent Hooks
│   └── human_approval.rs      #   人工审批 Hook (PromptHook)
│
├── pipeline/                  # AIOps 管道
│   ├── aiops.rs               #   Plan-Execute-Replan 循环编排
│   └── plan_types.rs          #   结构化计划类型 (JsonSchema)
│
├── mcp/                       # MCP 集成
│   └── cls.rs                 #   腾讯 CLS MCP 客户端 (rmcp)
│
├── vector_store/              # 向量存储
│   ├── milvus_store.rs        #   Milvus 初始化
│   └── embedding.rs           #   DashScope Embedding 适配
│
└── api/                       # HTTP API
    ├── mod.rs                 #   Router 组装 + CORS
    ├── chat.rs                #   POST /api/chat
    ├── chat_stream.rs         #   POST /api/chat_stream (SSE)
    ├── upload.rs              #   POST /api/upload
    └── ai_ops.rs              #   POST /api/ai_ops
```

## API 端点

### POST /api/chat

同步对话。

```json
// 请求
{ "id": "session-1", "question": "当前有哪些告警？" }

// 响应
{ "answer": "当前有以下活动告警：..." }
```

### POST /api/chat_stream

SSE 流式对话。

```json
// 请求
{ "id": "session-1", "question": "查询日志" }

// SSE 事件流
data: 当前正在...
event: tool_call
data: {"name":"query_prometheus_alerts",...}
data: 根据告警信息...
event: done
data: [DONE]
```

### POST /api/upload

文件上传 + 知识索引（支持 .txt / .md）。

```
// multipart/form-data
file: <文件>

// 响应
{ "file_name": "告警处理手册.md", "file_path": "./uploads/告警处理手册.md", "file_size": 2048 }
```

### POST /api/ai_ops

AIOps 自动告警分析。

```json
// 请求
{}

// 响应
{ "result": "根因分析...", "detail": ["--- 迭代 1 ---\n...", "--- 迭代 2 ---\n..."] }
```

## 快速开始

### 1. 前置依赖

- Rust 1.85+ (edition 2024)
- Milvus 2.5+ (本地 Docker 或远程)
- Prometheus (可选)
- MySQL (可选)
- 腾讯云 CLS MCP 服务 (可选)

### 2. 启动 Milvus

```bash
# 使用 docker-compose (需要 milvus 的 docker-compose.yml)
docker compose up -d
```

### 3. 配置环境变量

```bash
cp .env.example .env
# 编辑 .env，填入 API Key
```

必填项：

| 环境变量 | 说明 |
|---------|------|
| `VOLCENGINE_API_KEY` | 火山引擎 API Key (DeepSeek) |
| `DASHSCOPE_API_KEY` | 阿里云 DashScope API Key (Embedding) |

可选项（含默认值）：

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `VOLCENGINE_BASE_URL` | `https://ark.cn-beijing.volces.com/api/v3` | Volcengine API 地址 |
| `DEEPSEEK_THINK_MODEL` | `deepseek-v3-241226` | Think 模型 (计划/重规划) |
| `DEEPSEEK_QUICK_MODEL` | `deepseek-v3-241226` | Quick 模型 (执行/对话) |
| `DASHSCOPE_BASE_URL` | `https://dashscope.aliyuncs.com/compatible-mode/v1` | DashScope API 地址 |
| `DASHSCOPE_EMBEDDING_MODEL` | `text-embedding-v4` | Embedding 模型 |
| `MILVUS_URL` | `http://localhost:19530` | Milvus 地址 |
| `MILVUS_DATABASE` | `agent` | Milvus 数据库名 |
| `MILVUS_COLLECTION` | `biz` | Milvus 集合名 |
| `PROMETHEUS_URL` | `http://127.0.0.1:9090` | Prometheus 地址 |
| `MYSQL_DSN` | - | MySQL 连接字符串 |
| `MCP_URL` | - | MCP SSE 服务端地址 |
| `FILE_DIR` | `./uploads` | 文件上传目录 |
| `SERVER_PORT` | `6872` | HTTP 服务端口 |

### 4. 构建并运行

```bash
cargo run
```

### 5. 测试

```bash
# 同步对话
curl -X POST http://localhost:6872/api/chat \
  -H "Content-Type: application/json" \
  -d '{"id":"test","question":"当前有什么告警？"}'

# 流式对话
curl -X POST http://localhost:6872/api/chat_stream \
  -H "Content-Type: application/json" \
  -d '{"id":"test","question":"查询当前系统状态"}'

# AIOps 分析
curl -X POST http://localhost:6872/api/ai_ops \
  -H "Content-Type: application/json" \
  -d '{}'
```

## 架构设计

### Chat Agent

```
用户问题 → Agent (rig AgentBuilder)
            ├── preamble (系统提示)
            ├── dynamic_context(3, index) → Milvus RAG 自动检索
            ├── tools: [PrometheusAlerts, CurrentTime]
            ├── memory: InMemoryConversationMemory (6 条滑动窗口)
            └── Prompt → LLM → 工具调用 → 结果 → LLM → 最终回答
```

### AIOps Pipeline

```
告警信息 → Plan Extractor (Think 模型) → AioPsPlan
                                           ↓
                                       Executor Agent (Quick 模型 + Tools)
                                           ↓
                                       Replan Extractor (Think 模型) → InvestigationResult
                                           ↓
                                    needs_replan? → YES → 回到 Executor
                                                  → NO  → AioPsReport
```

### 功能映射 (Go Eino → Rust rig)

| Go (SuperBizAgent) | Rust (devOpsAgent) |
|---------------------|---------------------|
| `eino.ReactAgent` | `rig::AgentBuilder.build()` |
| `eino.tool.InvokableTool` | `rig::Tool` trait |
| `utils.InferOptionableTool` | `#[rig_tool]` derive 或手动 `impl Tool` |
| `planexecute.New()` | `Extractor<AioPsPlan>` + 循环编排函数 |
| `eino.SimpleMemory` | `rig::InMemoryConversationMemory` |
| `milvus.Retriever` | `rig::dynamic_context(3, index)` |
| `mcp-go.GetTools()` | `rig::McpClientHandler` + `rmcp` |
| `eino.Callback` | `rig::PromptHook` |
| GoFrame HTTP | axum |

## 运行单元测试

```bash
cargo test
```

当前测试覆盖：
- `tools/prometheus` — 告警简化、去重、持续时间计算
- `hooks/human_approval` — Hook 审批逻辑

## License

MIT
