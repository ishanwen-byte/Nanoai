//! # NanoAI - 轻量级 LLM 客户端库
//!
//! NanoAI 是一个专为 OpenRouter API 设计的轻量级 Rust 客户端库，
//! 提供简洁的接口来与各种大语言模型进行交互。
//!
//! ## 主要特性
//!
//! - 🚀 异步支持：基于 tokio 的完全异步实现
//! - 🔄 流式响应：支持实时流式文本生成
//! - 📊 统计信息：详细的请求统计和性能监控
//! - 🔧 灵活配置：支持环境变量和 Builder 模式配置
//! - 🛡️ 错误处理：完善的错误类型和重试机制
//! - 🎯 函数式设计：遵循 Rust 函数式编程最佳实践
//!
//! ## 快速开始
//!
//! ```rust
//! use nanoai::{Config, LLMClient};
//!
//! #[tokio::main]
//! async fn main() -> nanoai::Result<()> {
//!     let config = Config::from_env()?;
//!     let client = LLMClient::new(config);
//!     
//!     let response = client.generate("Hello, world!").await?;
//!     println!("Response: {}", response);
//!     
//!     Ok(())
//! }
//! ```

use futures::{Stream, StreamExt};
use log::{debug, error, info, warn};
use nanorand::{Rng, WyRand};
use reqwest::{
    Client as ReqwestClient,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::sleep;

// ================================================================================================
// 错误处理模块
// ================================================================================================

/// NanoAI 库的统一错误类型
/// 
/// 提供了完整的错误分类，便于上层应用进行精确的错误处理
#[derive(Debug, Error)]
pub enum NanoError {
    /// HTTP 请求相关错误
    #[error("HTTP请求失败: {0}")]
    Http(#[from] reqwest::Error),
    
    /// JSON 序列化/反序列化错误
    #[error("JSON处理错误: {0}")]
    Json(#[from] serde_json::Error),
    
    /// API 服务端错误
    #[error("API错误: {0}")]
    Api(String),
    
    /// 请求超时错误
    #[error("请求超时")]
    Timeout,
    
    /// 响应内容为空
    #[error("响应内容为空")]
    NoContent,
    
    /// 流处理相关错误
    #[error("流处理错误: {0}")]
    StreamError(String),
    
    /// API 请求频率限制
    #[error("请求频率超限: {0}")]
    RateLimit(String),
    
    /// 身份验证失败
    #[error("身份验证失败: {0}")]
    Auth(String),
    
    /// 指定的模型不存在
    #[error("模型不存在: {0}")]
    ModelNotFound(String),
    
    /// 请求参数无效
    #[error("请求参数无效: {0}")]
    InvalidRequest(String),
    
    /// 配置相关错误
    #[error("配置错误: {0}")]
    Config(String),
}

/// 库的统一 Result 类型
pub type Result<T> = std::result::Result<T, NanoError>;

// ================================================================================================
// 数据结构模块
// ================================================================================================

/// 聊天消息结构
/// 
/// 表示对话中的单条消息，包含角色和内容信息
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Message {
    /// 消息角色："system", "user", "assistant"
    pub role: String,
    /// 消息内容
    pub content: String,
}

/// 请求统计信息
/// 
/// 记录 API 请求的详细统计数据，用于性能监控和分析
#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    /// 请求耗时（毫秒）
    pub duration_ms: u64,
    /// 输入 token 数量
    pub prompt_tokens: Option<u32>,
    /// 输出 token 数量
    pub completion_tokens: Option<u32>,
    /// 总 token 数量
    pub total_tokens: Option<u32>,
    /// 使用的模型名称
    pub model: String,
    /// 请求时间戳
    pub timestamp: Option<std::time::SystemTime>,
}

/// 带统计信息的响应结果
/// 
/// 包含生成的内容和详细的请求统计信息
#[derive(Debug)]
pub struct ResponseWithStats {
    /// 生成的文本内容
    pub content: String,
    /// 请求统计信息
    pub stats: RequestStats,
}

// ================================================================================================
// 配置模块
// ================================================================================================

/// LLM 客户端配置
/// 
/// 包含所有必要的配置参数，支持 Builder 模式和环境变量配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 模型名称
    model: String,
    /// 系统消息
    system_message: String,
    /// 温度参数 (0.0-2.0)
    temperature: f32,
    /// Top-p 参数 (0.0-1.0)
    top_p: f32,
    /// 最大生成 token 数
    max_tokens: u32,
    /// 请求超时时间
    timeout: Duration,
    /// 重试次数
    retries: usize,
    /// 重试间隔
    retry_delay: Duration,
    /// API 基础 URL
    api_base: String,
    /// API 密钥
    api_key: String,
    /// 随机种子
    random_seed: Option<u64>,
}

impl Default for Config {
    /// 创建默认配置
    /// 
    /// 使用 DeepSeek 免费模型作为默认选择
    fn default() -> Self {
        Self {
            model: "tngtech/deepseek-r1t2-chimera:free".into(),
            system_message: "You are a helpful AI assistant.".into(),
            temperature: 0.7,
            top_p: 1.0,
            max_tokens: 4096,
            timeout: Duration::from_secs(60),
            retries: 3,
            retry_delay: Duration::from_millis(1000),
            api_base: "https://openrouter.ai/api/v1".into(),
            api_key: String::new(),
            random_seed: None,
        }
    }
}

/// 生成 Config Builder 方法的宏
/// 
/// 自动生成 `with_field_name` 形式的 builder 方法
macro_rules! config_builder {
    ($field:ident, $type:ty) => {
        paste::paste! {
            /// 设置配置字段
            pub fn [<with_ $field>](self, $field: $type) -> Self {
                Self { $field, ..self }
            }
        }
    };
    ($field:ident, $type:ty, $transform:expr) => {
        paste::paste! {
            /// 设置配置字段（带转换）
            pub fn [<with_ $field>](self, $field: $type) -> Self {
                Self { $field: $transform($field), ..self }
            }
        }
    };
}

impl Config {
    /// 从环境变量创建配置
    /// 
    /// 自动读取 .env 文件和环境变量：
    /// - `OPENROUTER_API_KEY` 或 `API_KEY`：API 密钥
    /// - `OPENROUTER_MODEL` 或 `MODEL`：模型名称
    pub fn from_env() -> Result<Self> {
        // 尝试加载 .env 文件
        if std::path::Path::new(".env").exists() {
            if let Ok(content) = std::fs::read_to_string(".env") {
                content
                    .lines()
                    .filter_map(|line| line.split_once('='))
                    .for_each(|(key, value)| {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        unsafe {
                            std::env::set_var(key, value);
                        }
                    });
            }
        }

        let api_key = std::env::var("OPENROUTER_API_KEY")
            .or_else(|_| std::env::var("API_KEY"))
            .map_err(|_| {
                NanoError::Config(
                    "No OpenRouter API key found in environment variables".to_string(),
                )
            })?;

        let model = std::env::var("OPENROUTER_MODEL")
            .or_else(|_| std::env::var("MODEL"))
            .unwrap_or_else(|_| "tngtech/deepseek-r1t2-chimera:free".to_string());

        Ok(Self::default().with_api_key(api_key).with_model(model))
    }

    // 使用宏生成 builder 方法
    config_builder!(api_base, String);
    config_builder!(model, String);
    config_builder!(api_key, String);
    config_builder!(temperature, f32);
    config_builder!(top_p, f32);
    config_builder!(max_tokens, u32);
    config_builder!(random_seed, u64, Some);

    /// 自动生成随机种子
    /// 
    /// 使用高性能的 WyRand 算法生成随机种子
    pub fn with_random_seed_auto(self) -> Self {
        let mut rng = WyRand::new();
        Self {
            random_seed: Some(rng.generate::<u64>()),
            ..self
        }
    }
}

// ================================================================================================
// API 响应结构模块
// ================================================================================================

/// API 完整响应结构
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
    usage: Option<Usage>,
    model: Option<String>,
}

/// API 响应选择项
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionChoice {
    message: CompletionMessage,
    finish_reason: Option<String>,
}

/// API 响应消息
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionMessage {
    content: String,
    role: Option<String>,
}

/// API 使用统计
#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 流式响应结构
#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
    #[allow(dead_code)]
    model: Option<String>,
    #[allow(dead_code)]
    usage: Option<Usage>,
}

/// 流式响应选择项
#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: Option<u32>,
}

/// 流式响应增量数据
#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
    #[allow(dead_code)]
    role: Option<String>,
}

// ================================================================================================
// 核心客户端模块
// ================================================================================================

/// LLM 客户端
/// 
/// 提供与 OpenRouter API 交互的核心功能，支持同步和流式请求
#[derive(Debug, Clone)]
pub struct LLMClient {
    /// HTTP 客户端
    client: ReqwestClient,
    /// 客户端配置
    config: Config,
    /// HTTP 请求头
    headers: HeaderMap,
}

impl LLMClient {
    /// 创建新的 LLM 客户端
    /// 
    /// # 参数
    /// 
    /// * `config` - 客户端配置
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use nanoai::{Config, LLMClient};
    /// 
    /// let config = Config::default().with_api_key("your-api-key".to_string());
    /// let client = LLMClient::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        static INITIALIZED: std::sync::OnceLock<std::sync::Mutex<Vec<String>>> =
            std::sync::OnceLock::new();

        let models_mutex = INITIALIZED.get_or_init(|| std::sync::Mutex::new(Vec::new()));
        if let Ok(mut models) = models_mutex.lock()
            && !models.contains(&config.model)
        {
            info!("Initialized LLM with model: {}", config.model);
            models.push(config.model.clone());
        }

        let headers = build_headers(&config.api_key);

        Self {
            client: ReqwestClient::builder()
                .timeout(config.timeout)
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| ReqwestClient::new()),
            config,
            headers,
        }
    }

    /// 生成文本响应
    /// 
    /// 最简单的文本生成接口，使用默认系统消息
    /// 
    /// # 参数
    /// 
    /// * `prompt` - 用户输入提示
    /// 
    /// # 返回
    /// 
    /// 生成的文本内容
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_stats(prompt)
            .await
            .map(|response| response.content)
    }

    /// 生成文本响应（带统计信息）
    /// 
    /// # 参数
    /// 
    /// * `prompt` - 用户输入提示
    /// 
    /// # 返回
    /// 
    /// 包含生成内容和统计信息的响应
    pub async fn generate_with_stats(&self, prompt: &str) -> Result<ResponseWithStats> {
        let user_message = Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        };
        
        self.generate_with_context_stats(&self.config.system_message, &[user_message])
            .await
    }

    /// 使用上下文生成文本响应（带统计信息）
    /// 
    /// # 参数
    /// 
    /// * `system_msg` - 系统消息
    /// * `messages` - 对话历史消息
    /// 
    /// # 返回
    /// 
    /// 包含生成内容和统计信息的响应
    pub async fn generate_with_context_stats(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<ResponseWithStats> {
        let start_time = Instant::now();
        let prepared_messages = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, prepared_messages);
        let response = self.call_api_with_stats(params).await?;
        let duration = start_time.elapsed();

        Ok(ResponseWithStats {
            content: response.content,
            stats: RequestStats {
                duration_ms: duration.as_millis() as u64,
                prompt_tokens: response.stats.prompt_tokens,
                completion_tokens: response.stats.completion_tokens,
                total_tokens: response.stats.total_tokens,
                model: self.config.model.clone(),
                timestamp: Some(std::time::SystemTime::now()),
            },
        })
    }

    /// 使用上下文生成文本响应
    /// 
    /// # 参数
    /// 
    /// * `system_msg` - 系统消息
    /// * `messages` - 对话历史消息
    /// 
    /// # 返回
    /// 
    /// 生成的文本内容
    pub async fn generate_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<String> {
        let prepared_messages = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, prepared_messages);
        self.call_with_retry(&params).await
    }

    /// 生成流式文本响应
    /// 
    /// 返回一个异步流，可以实时接收生成的文本片段
    /// 
    /// # 参数
    /// 
    /// * `prompt` - 用户输入提示
    /// 
    /// # 返回
    /// 
    /// 文本片段的异步流
    pub async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let user_msg = message("user", prompt);
        let messages = prepare_messages(&self.config.system_message, &[user_msg])?;
        self.create_stream(messages).await
    }

    /// 使用上下文生成流式文本响应
    /// 
    /// # 参数
    /// 
    /// * `system_msg` - 系统消息
    /// * `messages` - 对话历史消息
    /// 
    /// # 返回
    /// 
    /// 文本片段的异步流
    pub async fn generate_stream_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let all_messages = prepare_messages(system_msg, messages)?;
        self.create_stream(all_messages).await
    }

    /// 创建流式响应
    /// 
    /// 内部方法，处理流式响应的创建和数据处理
    async fn create_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let params = build_params_stream(&self.config, &messages)?;
        let response = self.send_request(&params).await?;
        let response = check_response_status(response).await?;

        use futures::future;
        Ok(response
            .bytes_stream()
            .map(|chunk_result| {
                chunk_result
                    .map_err(|e| NanoError::StreamError(e.to_string()))
                    .and_then(|chunk| {
                        std::str::from_utf8(&chunk)
                            .map_err(|e| NanoError::StreamError(format!("Invalid UTF-8: {e}")))
                            .and_then(process_stream_chunk)
                    })
            })
            .filter(|result| {
                future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            }))
    }

    /// 带重试机制的 API 调用
    /// 
    /// 根据配置的重试次数和延迟进行自动重试
    async fn call_with_retry(&self, params: &Value) -> Result<String> {
        let mut retries_left = self.config.retries;
        
        loop {
            match self.call_api(params).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retries_left > 0 {
                        warn!(
                            "Request failed: {e}. Retrying in {:?}...",
                            self.config.retry_delay
                        );
                        sleep(self.config.retry_delay).await;
                        retries_left -= 1;
                    } else {
                        error!("All {} retries exhausted: {e}", self.config.retries);
                        return Err(e);
                    }
                }
            }
        }
    }

    /// 发送 HTTP 请求
    /// 
    /// 内部方法，处理实际的 HTTP 请求发送
    async fn send_request(&self, params: &Value) -> Result<reqwest::Response> {
        let endpoint = format!("{}/chat/completions", self.config.api_base);

        self.client
            .post(&endpoint)
            .headers(self.headers.clone())
            .json(params)
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    NanoError::Timeout
                } else {
                    e.into()
                }
            })
    }

    /// 调用 API（仅返回内容）
    async fn call_api(&self, params: &Value) -> Result<String> {
        debug!("API parameters: {params:?}");
        let response = self.send_request(params).await?;
        handle_response(response).await
    }

    /// 调用 API（返回内容和统计信息）
    async fn call_api_with_stats(&self, params: Value) -> Result<ResponseWithStats> {
        let response = self.send_request(&params).await?;
        handle_response_with_stats(response).await
    }
}

// ================================================================================================
// 工具函数模块
// ================================================================================================

/// 准备消息列表
/// 
/// 将系统消息和用户消息合并为完整的消息列表
/// 
/// # 参数
/// 
/// * `system_msg` - 系统消息内容
/// * `messages` - 用户消息列表
/// 
/// # 返回
/// 
/// 包含系统消息的完整消息列表
fn prepare_messages(system_msg: &str, messages: &[Message]) -> Result<Vec<Message>> {
    let system_message = Message {
        role: "system".into(),
        content: system_msg.into(),
    };
    
    Ok([system_message]
        .into_iter()
        .chain(messages.iter().cloned())
        .collect())
}

/// 构建统一的 API 参数
/// 
/// 根据配置和消息构建 API 请求参数
fn build_params_unified<T>(config: &Config, messages: T, stream: bool) -> Value
where
    T: Into<Vec<Message>>,
{
    let mut params = serde_json::json!({
        "model": config.model,
        "messages": messages.into(),
        "stream": stream,
        "temperature": config.temperature,
        "top_p": config.top_p,
        "max_tokens": config.max_tokens,
    });

    if let Some(seed) = config.random_seed {
        params["seed"] = seed.into();
    }

    params
}

/// 构建非流式 API 参数
fn build_params(config: &Config, messages: Vec<Message>) -> Value {
    build_params_unified(config, messages, false)
}

/// 构建流式 API 参数
fn build_params_stream(config: &Config, messages: &[Message]) -> Result<Value> {
    Ok(build_params_unified(config, messages.to_vec(), true))
}

/// 构建 HTTP 请求头
/// 
/// 创建包含认证和内容类型的 HTTP 头
/// 
/// # 参数
/// 
/// * `api_key` - API 密钥
/// 
/// # 返回
/// 
/// 配置好的 HTTP 头映射
fn build_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // 添加 Bearer 认证头，符合 OAuth 2.0 规范
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
    );

    // 指定请求体内容类型为 JSON
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );

    headers
}

/// 检查响应状态码
/// 
/// 根据 HTTP 状态码返回相应的错误类型
async fn check_response_status(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await?;
        return Err(match status.as_u16() {
            401 => NanoError::Auth(error_text),
            404 => NanoError::ModelNotFound(error_text),
            429 => NanoError::RateLimit(error_text),
            400 => NanoError::InvalidRequest(error_text),
            _ => NanoError::Api(format!("HTTP {}: {}", status, error_text)),
        });
    }
    Ok(response)
}

/// 提取响应内容的宏
/// 
/// 从 API 响应中提取文本内容
macro_rules! extract_content {
    ($completion:expr) => {
        $completion
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(NanoError::NoContent)
    };
}

/// 构建请求统计信息的宏
/// 
/// 从 API 响应中提取统计信息
macro_rules! build_stats {
    ($completion:expr) => {
        RequestStats {
            duration_ms: 0, // 由调用方根据实际请求耗时设置
            prompt_tokens: $completion.usage.as_ref().map(|u| u.prompt_tokens),
            completion_tokens: $completion.usage.as_ref().map(|u| u.completion_tokens),
            total_tokens: $completion.usage.as_ref().map(|u| u.total_tokens),
            model: String::new(), // 由调用方设置实际使用的模型名称
            timestamp: Some(std::time::SystemTime::now()),
        }
    };
}

/// 处理普通响应
/// 
/// 解析 API 响应并提取文本内容
async fn handle_response(response: reqwest::Response) -> Result<String> {
    let response = check_response_status(response).await?;
    let completion: CompletionResponse = response.json().await?;
    extract_content!(completion)
}

/// 处理带统计信息的响应
/// 
/// 解析 API 响应并提取内容和统计信息
async fn handle_response_with_stats(response: reqwest::Response) -> Result<ResponseWithStats> {
    let response = check_response_status(response).await?;
    let completion: CompletionResponse = response.json().await?;
    let content = extract_content!(completion)?;
    let stats = build_stats!(completion);
    Ok(ResponseWithStats { content, stats })
}

/// 处理流式响应数据块
/// 
/// 解析 SSE (Server-Sent Events) 格式的流式数据
/// 
/// # 参数
/// 
/// * `text` - 原始文本数据
/// 
/// # 返回
/// 
/// 提取的文本内容片段
fn process_stream_chunk(text: &str) -> Result<String> {
    // 检查是否包含 SSE 数据标记
    if !text.contains("data: ") {
        return Ok(String::new());
    }

    // 逐行处理 SSE 数据
    text.lines()
        .filter(|line| line.starts_with("data: ") && !line.contains("[DONE]"))
        .find_map(|line| {
            line.strip_prefix("data: ")
                .and_then(|json_str| serde_json::from_str::<StreamResponse>(json_str).ok())
                .and_then(|stream_data| {
                    stream_data
                        .choices
                        .into_iter()
                        .next()
                        .and_then(|c| c.delta.content)
                })
        })
        .map(Ok)
        .unwrap_or_else(|| Ok(String::new()))
}

/// 创建消息的便捷函数
/// 
/// # 参数
/// 
/// * `role` - 消息角色
/// * `content` - 消息内容
/// 
/// # 返回
/// 
/// 新创建的消息实例
/// 
/// # 示例
/// 
/// ```rust
/// use nanoai::message;
/// 
/// let msg = message("user", "Hello, world!");
/// assert_eq!(msg.role, "user");
/// assert_eq!(msg.content, "Hello, world!");
/// ```
pub fn message(role: &str, content: &str) -> Message {
    Message {
        role: role.to_string(),
        content: content.to_string(),
    }
}

// ================================================================================================
// 测试模块
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 简化配置字段断言的宏
    macro_rules! assert_config_field {
        ($config:expr, $field:ident, $expected:expr) => {
            assert_eq!($config.$field, $expected);
        };
    }

    /// 简化参数断言的宏
    macro_rules! assert_param {
        ($params:expr, $key:expr, $expected:expr) => {
            assert_eq!($params[$key], $expected);
        };
        ($params:expr, $key:expr, float, $expected:expr) => {
            let value = $params[$key].as_f64().unwrap();
            assert!((value - $expected).abs() < 0.001);
        };
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_config_field!(config, model, "tngtech/deepseek-r1t2-chimera:free");
        assert_config_field!(config, temperature, 0.7);
        assert_config_field!(config, max_tokens, 4096);
        assert_config_field!(config, random_seed, None);
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = Config::default()
            .with_model("gpt-4".to_string())
            .with_api_key("test-key".to_string())
            .with_temperature(0.5)
            .with_random_seed(12345);
        assert_config_field!(config, model, "gpt-4");
        assert_config_field!(config, api_key, "test-key");
        assert_config_field!(config, temperature, 0.5);
        assert_config_field!(config, random_seed, Some(12345));
    }

    #[test]
    fn test_config_random_seed_auto() {
        let config1 = Config::default().with_random_seed_auto();
        let config2 = Config::default().with_random_seed_auto();
        assert!(config1.random_seed.is_some());
        assert!(config2.random_seed.is_some());
        assert_ne!(config1.random_seed, config2.random_seed);
    }

    #[test]
    fn test_message_creation() {
        let msg = message("user", "Hello, world!");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_prepare_messages() {
        let user_messages = vec![message("user", "Hello"), message("assistant", "Hi there!")];
        let result = prepare_messages("You are helpful", &user_messages).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].role, "system");
        assert_eq!(result[0].content, "You are helpful");
        assert_eq!(result[1].role, "user");
        assert_eq!(result[2].role, "assistant");
    }

    #[test]
    fn test_build_params_without_seed() {
        let config = Config::default();
        let messages = vec![message("user", "test")];
        let params = build_params(&config, messages);
        assert_param!(params, "model", "tngtech/deepseek-r1t2-chimera:free");
        assert_param!(params, "stream", false);
        assert_param!(params, "max_tokens", 4096);
        assert_param!(params, "temperature", float, 0.7);
        assert!(params.get("seed").is_none());
    }

    #[test]
    fn test_build_params_with_seed() {
        let config = Config::default().with_random_seed(42);
        let messages = vec![message("user", "test")];
        let params = build_params(&config, messages);
        assert_param!(params, "seed", 42);
    }

    #[test]
    fn test_build_params_openrouter() {
        let config = Config::default().with_model("openai/gpt-4".to_string());
        let messages = vec![message("user", "test")];
        let params = build_params(&config, messages);
        assert_param!(params, "model", "openai/gpt-4");
        assert_param!(params, "max_tokens", 4096);
        assert_param!(params, "temperature", float, 0.7);
        assert_param!(params, "top_p", 1.0);
    }

    #[test]
    fn test_build_headers() {
        let headers = build_headers("test-api-key");
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
        let auth_value = headers.get("authorization").unwrap().to_str().unwrap();
        assert_eq!(auth_value, "Bearer test-api-key");
    }

    #[tokio::test]
    async fn test_llm_client_creation() {
        let config = Config::default().with_api_key("test-key".to_string());
        let client = LLMClient::new(config.clone());
        assert_config_field!(client.config, api_key, "test-key");
    }
}
