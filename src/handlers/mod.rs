use crate::models::ModelsResponse;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use lazy_static::lazy_static;
use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Map, Value};
use std::sync::Arc;

lazy_static! {
    static ref HEADER_AUTHORIZATION: HeaderName = HeaderName::from_static("authorization");
    static ref HEADER_CONTENT_TYPE: HeaderName = HeaderName::from_static("content-type");
    static ref HEADER_USER_PROJECT: HeaderName = HeaderName::from_static("x-goog-user-project");
    static ref CONTENT_TYPE_JSON: HeaderValue = HeaderValue::from_static("application/json");
    static ref GLOBAL: String = "global".to_owned();
}

const MODLES_URL: &str =
    "https://us-central1-aiplatform.googleapis.com/v1beta1/publishers/google/models";

/// 根路径健康检查
pub async fn root() -> &'static str {
    "Hello, this is Simple Vertex Bridge! UwU"
}

/// 聊天完成接口 - GET/POST
///
/// 直接透传 Vertex AI 的响应,包括所有响应头
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<Response, StatusCode> {
    use axum::body::Body;

    // 1. 获取认证令牌
    let auth_header = state.token_manager.authorization().await.map_err(|e| {
        tracing::error!("Failed to get authorization token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let request_body: Map<String, Value> = serde_json::from_str(&body).map_err(|e| {
        tracing::error!("Failed to deserialize body: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 2. 构建 Vertex AI URL
    let mut location = &state.config.location;
    let project_id = state.config.project_id;
    let endpoint_id = state.config.endpoint_id;
    if let Some(model) = request_body.get("model") {
        let model_id = model.as_str().unwrap_or("");
        if model_id.contains("gemini-3") {
            location = &GLOBAL;
        }
    }
    let url = if location.eq("global") {
        format!(
            "https://aiplatform.googleapis.com/v1beta1/projects/{project_id}/locations/{location}/endpoints/{endpoint_id}/chat/completions"
        )
    } else {
        format!(
            "https://{location}-aiplatform.googleapis.com/v1beta1/projects/{project_id}/locations/{location}/endpoints/{endpoint_id}/chat/completions"
        )
    };

    tracing::debug!("Forwarding chat completion request to: {}", url);

    // 3. 构建请求,先设置我们的认证头
    let mut request_builder = state
        .http_client
        .post(&url)
        .header(AUTHORIZATION, auth_header)
        .header(HEADER_USER_PROJECT.clone(), project_id)
        .header(CONTENT_TYPE, CONTENT_TYPE_JSON.clone());

    // 4. 转发客户端的其他请求头(排除敏感头和我们自己设置的头)
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        // 跳过这些头:我们会自己设置或者不应该转发
        if key_str == "host"
            || key_str == "authorization"
            || key_str == "content-length"
            || key_str == "content-type"
            || key_str.starts_with("x-goog-")
        {
            continue;
        }
        request_builder = request_builder.header(key, value);
    }

    // 5. 发送请求
    let response = request_builder.body(body).send().await.map_err(|e| {
        tracing::error!("Failed to forward request to Vertex AI: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    // 6. 检查响应状态
    let status = response.status();
    if !status.is_success() {
        tracing::error!("Vertex AI returned error status: {}", status);
        // 即使是错误也透传完整响应
    }

    // 7. 复制响应头
    let mut response_builder = Response::builder().status(status);

    // 复制所有响应头
    for (key, value) in response.headers() {
        response_builder = response_builder.header(key, value);
    }

    // 8. 直接透传响应体(支持流式和非流式)
    let body = Body::from_stream(response.bytes_stream());
    Ok(response_builder.body(body).unwrap())
}

/// 获取可用模型列表
///
/// 从 Vertex AI 获取可用模型并转换为 OpenAI 格式,使用缓存减少 API 调用
pub async fn models(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
) -> Result<Json<ModelsResponse>, StatusCode> {
    use crate::models::VertexModelsResponse;

    // 1. 先检查缓存
    if let Some(cached_models) = state.models_cache.get("vertex_models").await {
        tracing::debug!("Returning {} models from cache", cached_models.len());
        return Ok(Json(ModelsResponse {
            object: "list",
            data: cached_models,
        }));
    }

    tracing::debug!("Cache miss, fetching models from Vertex AI");

    // 2. 获取 GCP 访问令牌
    let auth_header = state.token_manager.authorization().await.map_err(|e| {
        tracing::error!("Failed to get authorization token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. 构建 Vertex AI API URL
    let project_id = state.config.project_id;
    let url = MODLES_URL;

    tracing::debug!("Requesting models from: {}", url);

    // 4. 请求 Vertex AI 模型列表
    let response = state
        .http_client
        .get(url)
        .header(AUTHORIZATION, auth_header)
        .header(HEADER_USER_PROJECT.clone(), project_id)
        .header(CONTENT_TYPE, CONTENT_TYPE_JSON.clone())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch models from Vertex AI: {:?}", e);
            tracing::error!("Error details: {}", e);

            // 检查是否是网络连接问题
            if e.is_connect() {
                tracing::error!("Connection error - check network connectivity");
            } else if e.is_timeout() {
                tracing::error!("Request timeout");
            } else if e.is_request() {
                tracing::error!("Request error");
            }

            StatusCode::BAD_GATEWAY
        })?;

    // 5. 检查响应状态
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error body".to_string());
        tracing::error!("Vertex AI returned error status {}: {}", status, error_body);
        return Err(StatusCode::BAD_GATEWAY);
    }

    // 6. 解析响应
    let vertex_response: VertexModelsResponse = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse Vertex AI response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 7. 转换为 OpenAI 格式并过滤
    let models: Vec<_> = vertex_response
        .publisher_models
        .into_iter()
        .filter(|m| m.should_include())
        .map(|m| m.to_openai_model())
        .collect();

    tracing::info!("Fetched {} models from Vertex AI", models.len());

    // 8. 缓存结果
    state.models_cache.invalidate_all();
    state
        .models_cache
        .insert("vertex_models".to_owned(), models.clone())
        .await;
    tracing::debug!("Models cached for 1 hour");

    Ok(Json(ModelsResponse {
        object: "list",
        data: models,
    }))
}
