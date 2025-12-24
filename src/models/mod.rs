use serde::{Deserialize, Serialize};

/// OpenAI 消息格式
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// OpenAI 聊天完成请求
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

/// OpenAI 模型信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

/// OpenAI 模型列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: &'static str,
    pub data: Vec<Model>,
}

// ============= Vertex AI 相关结构 =============

/// Vertex AI 模型列表响应
#[derive(Debug, Deserialize)]
pub struct VertexModelsResponse {
    #[serde(rename = "publisherModels")]
    pub publisher_models: Vec<VertexModel>,
}

/// Vertex AI 单个模型信息
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct VertexModel {
    pub name: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
    #[serde(rename = "launchStage")]
    pub launch_stage: Option<String>,
    #[serde(rename = "openSourceCategory")]
    pub open_source_category: Option<String>,
}

impl VertexModel {
    /// 从 Vertex AI 模型转换为 OpenAI 模型格式
    ///
    /// 提取模型名称并创建 OpenAI 兼容的模型对象
    pub fn to_openai_model(&self) -> Model {
        // 从 "publishers/google/models/gemma-2b" 提取 "google/gemma-2b"
        // 需要第二个部分(publisher)和第四个部分(model name)
        let parts: Vec<&str> = self.name.split('/').collect();
        let model_id = if parts.len() >= 4 {
            format!("{}/{}", parts[1], parts[3])
        } else {
            // 如果格式不符合预期,使用最后一部分
            parts.last().unwrap_or(&"unknown").to_string()
        };

        Model {
            id: model_id,
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp(),
            owned_by: "google".to_string(),
        }
    }

    /// 判断模型是否应该被包含在列表中
    ///
    /// 过滤条件:
    /// - 只包含 GA (正式发布) 和 PUBLIC_PREVIEW (公开预览) 的模型
    pub fn should_include(&self) -> bool {
        self.name.contains("gemini")
            && matches!(self.launch_stage.as_deref(), Some("GA") | Some("PUBLIC_PREVIEW"))
    }
}
