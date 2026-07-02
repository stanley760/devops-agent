//! Pipeline 编排 — 将 AgentRunner/Subagent 组合为工作流
//!
//! 提供 AgentOp、SubagentOp 等 pipeline 基本单元，
//! 以及 Pipeline builder 用于便捷组合。

use std::future::Future;

use devops_agent_core::error::SubagentError;
use devops_agent_core::traits::{AgentRunner, Subagent};

/// 将 AgentRunner 包装为可调用的 pipeline Op
pub struct AgentOp<R: AgentRunner> {
    runner: R,
}

impl<R: AgentRunner> AgentOp<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    /// 执行 agent runner
    pub async fn call(&self, input: R::Input) -> Result<R::Output, R::Error> {
        self.runner.run(input).await
    }
}

/// 将 Subagent 包装为可调用的 pipeline Op
pub struct SubagentOp<S: Subagent> {
    subagent: S,
    task_template: String,
}

impl<S: Subagent> SubagentOp<S> {
    pub fn new(subagent: S, task_template: &str) -> Self {
        Self {
            subagent,
            task_template: task_template.to_string(),
        }
    }

    /// 执行 subagent 委托
    pub async fn call(&self, input: &str) -> Result<String, SubagentError> {
        let task = self.task_template.replace("{input}", input);
        self.subagent.delegate(&task).await
    }
}

/// 异步 map 操作
pub struct MapOp<F, I, O> {
    f: F,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<F, I, O> MapOp<F, I, O>
where
    F: Fn(I) -> O + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn call(&self, input: I) -> O {
        (self.f)(input)
    }
}

/// 异步 then 操作
pub struct ThenOp<F, I, Fut> {
    f: F,
    _phantom: std::marker::PhantomData<(I, Fut)>,
}

impl<F, I, Fut> ThenOp<F, I, Fut>
where
    F: Fn(I) -> Fut + Send + Sync,
    Fut: Future + Send,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn call(&self, input: I) -> Fut::Output {
        (self.f)(input).await
    }
}

/// Pipeline builder — 便捷组合 agent 操作
pub struct Pipeline;

impl Pipeline {
    /// 从 AgentRunner 创建 pipeline 入口
    pub fn agent<R: AgentRunner>(runner: R) -> AgentOp<R> {
        AgentOp::new(runner)
    }

    /// 从 Subagent 创建 pipeline 入口
    pub fn subagent<S: Subagent>(subagent: S, template: &str) -> SubagentOp<S> {
        SubagentOp::new(subagent, template)
    }

    /// 创建同步 map 操作
    pub fn map<F, I, O>(f: F) -> MapOp<F, I, O>
    where
        F: Fn(I) -> O + Send + Sync,
    {
        MapOp::new(f)
    }

    /// 创建异步 then 操作
    pub fn then<F, I, Fut>(f: F) -> ThenOp<F, I, Fut>
    where
        F: Fn(I) -> Fut + Send + Sync,
        Fut: Future + Send,
    {
        ThenOp::new(f)
    }
}
