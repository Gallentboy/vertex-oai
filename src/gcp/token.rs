use axum::http::Extensions;
use google_cloud_auth::credentials::{CacheableResource, Credentials};
use reqwest::header::HeaderValue;
use std::sync::Arc;
use tokio::sync::RwLock;

/// GCP 令牌管理器
///
/// 使用官方 google-cloud-auth 库实现,支持多种认证方式:
/// - Application Default Credentials (ADC)
/// - 服务账号密钥文件 (GOOGLE_APPLICATION_CREDENTIALS)
/// - 元数据服务器 (GCE/GKE/Cloud Run)
/// - gcloud CLI 凭据
#[derive(Clone)]
pub struct TokenManager {
    credentials: Arc<RwLock<Credentials>>,

    auth: Arc<RwLock<HeaderValue>>,
}

impl TokenManager {
    /// 创建新的令牌管理器
    ///
    /// 自动按以下顺序查找凭据:
    /// 1. GOOGLE_APPLICATION_CREDENTIALS 环境变量指定的服务账号文件
    /// 2. ~/.config/gcloud/application_default_credentials.json (gcloud ADC)
    /// 3. 元数据服务器 (在 GCP 环境中)
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            credentials: Arc::new(RwLock::new(
                google_cloud_auth::credentials::Builder::default().build()?,
            )),
            auth: Arc::new(RwLock::new(HeaderValue::from_static(""))),
        })
    }

    pub async fn authorization(&self) -> Result<HeaderValue, Box<dyn std::error::Error>> {
        match self
            .credentials
            .read()
            .await
            .headers(Extensions::new())
            .await?
        {
            CacheableResource::NotModified => {
                Ok(self.auth.read().await.clone())
            }
            CacheableResource::New { entity_tag: _, data } => {
                let option = data.get("authorization");
                let value = option
                    .map(|v| v.to_str().unwrap_or("").to_owned())
                    .unwrap_or(String::new());
                let hv = HeaderValue::from_str(value.as_str())?;
                *self.auth.write().await = hv.clone();
                Ok(hv)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要 GCP 凭据才能运行
    async fn test_token_manager_creation() {
        let manager = TokenManager::new().await.unwrap();
        println!("==> {:?}", manager.authorization().await.unwrap());
        println!("==> {:?}", manager.authorization().await.unwrap());
    }
}
