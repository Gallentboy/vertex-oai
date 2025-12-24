use crate::gcp::TokenManager;
use crate::models::Model;
use moka::future::Cache;
use std::process::exit;

/// 应用配置
#[derive(Clone)]
pub struct Config {
    pub location: String,
    pub endpoint_id: &'static str,
    pub project_id: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            location: "us-central1".to_string(),
            endpoint_id: "openapi",
            project_id: "",
        }
    }
}

/// 应用状态
///
/// 存储应用级别的共享状态,如 HTTP 客户端、配置等
#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub token_manager: TokenManager,
    pub config: Config,
    pub models_cache: Cache<String, Vec<Model>>,
}

impl AppState {
    /// 创建新的应用状态实例
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 创建 HTTP 客户端 - 优化配置
        let http_client = reqwest::Client::builder()
            // 超时配置
            .timeout(std::time::Duration::from_secs(60)) // 总超时时间
            .connect_timeout(std::time::Duration::from_secs(10)) // 连接超时
            
            // 连接池配置
            .pool_max_idle_per_host(10) // 每个主机最大空闲连接数
            .pool_idle_timeout(std::time::Duration::from_secs(90)) // 空闲连接超时
            
            // TCP 配置
            .tcp_keepalive(std::time::Duration::from_secs(60)) // TCP keep-alive
            .tcp_nodelay(true) // 禁用 Nagle 算法,减少延迟
            
            // HTTP/2 配置 - Vertex AI 支持 HTTP/2
            .http2_prior_knowledge() // 优先使用 HTTP/2
            .http2_adaptive_window(true) // 自适应窗口大小
            .http2_keep_alive_interval(std::time::Duration::from_secs(30)) // HTTP/2 keep-alive
            .http2_keep_alive_timeout(std::time::Duration::from_secs(20))
            .http2_keep_alive_while_idle(true) // 空闲时也保持 keep-alive
            
            // 其他优化
            .gzip(true) // 启用 gzip 压缩
            .brotli(true) // 启用 brotli 压缩
            .deflate(true) // 启用 deflate 压缩
            
            .build()?;

        // 创建令牌管理器
        let token_manager = TokenManager::new().await?;

        // 从环境变量创建配置,未设置时使用默认值
        let config = Config {
            location: std::env::var("GCP_LOCATION").unwrap_or_else(|_| "global".to_string()),
            endpoint_id: "openapi",
            project_id: std::env::var("GCP_PROJECT_ID")
                .unwrap_or_else(|_| {
                    tracing::error!("请设置GCP_PROJECT_ID");
                    exit(1);
                })
                .leak(),
        };

        // 打印配置信息
        tracing::info!("========================================");
        tracing::info!("GCP Configuration:");
        tracing::info!("  Location:    {}", config.location);
        tracing::info!("  Endpoint ID: {}", config.endpoint_id);
        tracing::info!("  Project ID:  {}", config.project_id);
        tracing::info!("========================================");

        // 创建模型缓存 - 1小时过期
        let models_cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(3600)) // 1小时
            .build();

        Ok(Self {
            http_client,
            token_manager,
            config,
            models_cache,
        })
    }
}
