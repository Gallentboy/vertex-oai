use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::state::AppState;

/// 创建应用路由
/// 
/// 包含所有 API 端点:
/// - `/` - 健康检查
/// - `/chat/completions` - 聊天完成接口 (GET/POST)
/// - `/v1/chat/completions` - 聊天完成接口 (GET/POST)
/// - `/models` - 模型列表接口
/// - `/v1/models` - 模型列表接口
pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // 根路径
        .route("/", get(handlers::root))
        // 聊天完成接口 (支持 GET 和 POST)
        .route(
            "/chat/completions",
            get(handlers::chat_completions).post(handlers::chat_completions),
        )
        .route(
            "/v1/chat/completions",
            get(handlers::chat_completions).post(handlers::chat_completions),
        )
        // 模型列表接口
        .route("/models", get(handlers::models))
        .route("/v1/models", get(handlers::models))
        .with_state(state)
}
