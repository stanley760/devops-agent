//! AIOps Agent 构建 — 使用 devops-agent-agent 框架
//!
//! 业务特定的 preamble 和 agent 组装逻辑保留在此，
//! 通用的 Plan-Execute-Replan 循环逻辑在 devops-agent-agent crate 中。

use rig_core::client::CompletionClient;
use rig_core::providers;
use rig_core::providers::deepseek::CompletionModel as DeepSeekCompletionModel;

use devops_agent_agent::plan_execute::{PlanExecuteAgents, ReplanResult};

use crate::config::AppConfig;
use crate::plan_types::AioPsPlan;
use crate::tools::current_time::CurrentTimeTool;
use crate::tools::prometheus::PrometheusAlertsTool;

/// AIOps 系统 prompt — 计划阶段
const AIOPS_PLAN_PREAMBLE: &str = r#"你是一个专业的 SRE/AIOps 规划师。根据告警信息，生成结构化的调查计划。

你的职责:
1. 分析告警的严重程度和影响范围
2. 制定系统化的调查步骤
3. 每个步骤指定要使用的工具和期望发现

可用工具:
- query_prometheus_alerts: 查询 Prometheus 活动告警
- get_current_time: 获取当前时间（用于时间相关查询前获取时间戳）
- query_internal_docs: 查询内部文档知识库（告警处理手册、运维指南）

请严格按照 JSON schema 返回结构化计划。"#;

/// AIOps 系统 prompt — 执行阶段
const AIOPS_EXECUTE_PREAMBLE: &str = r#"你是一个专业的 SRE/AIOps 执行者。按照给定的调查计划，逐步执行调查步骤。

规则:
1. 严格按照计划中的步骤顺序执行
2. 每步调用指定工具并分析结果
3. 在查询时间相关数据前，先调用 get_current_time 获取当前时间
4. 查询日志时，需要指定 region 和 topic_id 参数
5. 严格遵守内部文档中的操作流程
6. 汇总所有发现，形成完整的调查报告"#;

/// AIOps 系统 prompt — 重规划阶段
const AIOPS_REPLAN_PREAMBLE: &str = r#"你是一个专业的 SRE/AIOps 评估者。评估执行结果，判断是否需要重新规划调查方向。

判断标准:
- 如果所有步骤已完成且有明确结论 → needs_replan = false
- 如果发现新的问题或需要进一步调查 → needs_replan = true，提供修订计划
- 如果执行未获得足够信息 → needs_replan = true，调整调查策略

请严格按照 JSON schema 返回评估结果。"#;

/// AIOps 各阶段 Agent 和 Extractor 的集合
///
/// 使用 devops-agent-agent 的 PlanExecuteAgents 类型，
/// 泛型参数为 DeepSeekCompletionModel 和 AioPsPlan。
pub type AiopsAgents = PlanExecuteAgents<DeepSeekCompletionModel, AioPsPlan>;

/// 构建 AIOps Agent 集合
pub fn build_aiops_agents(
    config: &AppConfig,
) -> Result<AiopsAgents, Box<dyn std::error::Error>> {
    // DeepSeek 客户端
    let ds_client = providers::deepseek::Client::builder()
        .api_key(&config.volcengine_api_key)
        .base_url(&config.volcengine_base_url)
        .build()?;

    // 计划提取器 (Think 模型)
    let plan_extractor = ds_client
        .extractor::<AioPsPlan>(&config.deepseek_think_model)
        .preamble(AIOPS_PLAN_PREAMBLE)
        .retries(1)
        .build();

    // 执行 Agent (Quick 模型 + Tools)
    let quick_model = ds_client.completion_model(&config.deepseek_quick_model);
    let executor_agent = rig_core::agent::AgentBuilder::new(quick_model)
        .name("aiops-executor")
        .preamble(AIOPS_EXECUTE_PREAMBLE)
        .tool(PrometheusAlertsTool::new(config.prometheus_url.clone()))
        .tool(CurrentTimeTool)
        .temperature(0.3)
        .default_max_turns(10)
        .build();

    // 重规划提取器 (Think 模型)
    let replan_extractor = ds_client
        .extractor::<ReplanResult<AioPsPlan>>(&config.deepseek_think_model)
        .preamble(AIOPS_REPLAN_PREAMBLE)
        .retries(1)
        .build();

    Ok(PlanExecuteAgents {
        planner: plan_extractor,
        executor: executor_agent,
        replanner: replan_extractor,
    })
}
