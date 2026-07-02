//! Agent 路由构建器
//!
//! Bevy 风格的"向 App 添加功能"模式，用于从 agent 构建完整的 axum Router。

use axum::Router;
use tower_http::cors::CorsLayer;

/// Agent 路由构建器
///
/// # Example
/// ```no_run
/// use devops_agent_server::router::AgentRouterBuilder;
///
/// let app = AgentRouterBuilder::new()
///     .cors(CorsLayer::permissive())
///     .build();
/// ```
pub struct AgentRouterBuilder {
    cors: Option<CorsLayer>,
}

impl AgentRouterBuilder {
    pub fn new() -> Self {
        Self { cors: None }
    }

    /// 设置 CORS 层
    pub fn cors(mut self, cors: CorsLayer) -> Self {
        self.cors = Some(cors);
        self
    }

    /// 构建最终的 Router
    ///
    /// 业务代码先用自己的 Router 添加路由，然后调用此方法应用 CORS 等中间件
    pub fn apply(self, router: Router) -> Router {
        let mut router = router;

        if let Some(cors) = self.cors {
            router = router.layer(cors);
        }

        router
    }
}

impl Default for AgentRouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
