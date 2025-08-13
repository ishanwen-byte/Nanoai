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
#[derive(Debug, Error)]
pub enum NanoError {
    #[error("HTTP请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON处理错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API错误: {0}")]
    Api(String),

    #[error("请求超时")]
    Timeout,

    #[error("响应内容为空")]
    NoContent,

    #[error("流处理错误: {0}")]
    StreamError(String),

    #[error("请求频率超限: {0}")]
    RateLimit(String),

    #[error("身份验证失败: {0}")]
    Auth(String),

    #[error("模型不存在: {0}")]
    ModelNotFound(String),

    #[error("请求参数无效: {0}")]
    InvalidRequest(String),

    #[error("配置错误: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, NanoError>;
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    pub duration_ms: u64,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub model: String,
    pub timestamp: Option<std::time::SystemTime>,
}

#[derive(Debug)]
pub struct ResponseWithStats {
    pub content: String,
    pub stats: RequestStats,
}

#[derive(Debug, Clone)]
pub struct Config {
    model: String,
    system_message: String,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
    timeout: Duration,
    retries: usize,
    retry_delay: Duration,
    api_base: String,
    api_key: String,
    random_seed: Option<u64>,
}

impl Default for Config {
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
            api_key: "".into(),
            random_seed: None,
        }
    }
}

// 宏：生成Config的builder方法
macro_rules! config_builder {
    ($field:ident, $type:ty) => {
        paste::paste! {
            pub fn [<with_ $field>](self, $field: $type) -> Self {
                Self { $field, ..self }
            }
        }
    };
    ($field:ident, $type:ty, $transform:expr) => {
        paste::paste! {
            pub fn [<with_ $field>](self, $field: $type) -> Self {
                Self { $field: $transform($field), ..self }
            }
        }
    };
}

impl Config {
    pub fn from_env() -> Result<Self> {
        if std::path::Path::new(".env").exists()
            && let Ok(content) = std::fs::read_to_string(".env")
        {
            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    unsafe {
                        std::env::set_var(key, value);
                    }
                }
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

    // 使用宏生成builder方法
    config_builder!(model, String);
    config_builder!(api_key, String);
    config_builder!(temperature, f32);
    config_builder!(random_seed, u64, Some);

    pub fn with_random_seed_auto(self) -> Self {
        let mut rng = WyRand::new();
        Self {
            random_seed: Some(rng.generate::<u64>()),
            ..self
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
    usage: Option<Usage>,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionChoice {
    message: CompletionMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionMessage {
    content: String,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
    #[allow(dead_code)]
    model: Option<String>,
    #[allow(dead_code)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
    #[allow(dead_code)]
    index: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LLMClient {
    client: ReqwestClient,
    config: Config,
    headers: HeaderMap,
}

impl LLMClient {
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
                .danger_accept_invalid_certs(false)
                .use_rustls_tls()
                .timeout(config.timeout)
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| ReqwestClient::new()),
            config,
            headers,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let response = self.generate_with_stats(prompt).await?;
        Ok(response.content)
    }

    pub async fn generate_with_stats(&self, prompt: &str) -> Result<ResponseWithStats> {
        self.generate_with_context_stats(
            &self.config.system_message,
            &[Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        )
        .await
    }

    pub async fn generate_with_context_stats(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<ResponseWithStats> {
        let start_time = Instant::now();
        let msgs = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, msgs);
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

    pub async fn generate_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<String> {
        let msgs = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, msgs);
        self.call_with_retry(&params).await
    }

    pub async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let user_msg = message("user", prompt);
        let messages = prepare_messages(&self.config.system_message, &[user_msg])?;
        self.create_stream(messages).await
    }

    pub async fn generate_stream_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let all_messages = prepare_messages(system_msg, messages)?;
        self.create_stream(all_messages).await
    }

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
            .map(|chunk_result| match chunk_result {
                Ok(chunk) => match std::str::from_utf8(&chunk) {
                    Ok(text) => process_stream_chunk(text),
                    Err(e) => Err(NanoError::StreamError(format!("Invalid UTF-8: {e}"))),
                },
                Err(e) => Err(NanoError::StreamError(e.to_string())),
            })
            .filter(|result| {
                future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            }))
    }

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

    async fn call_api(&self, params: &Value) -> Result<String> {
        debug!("API parameters: {params:?}");
        let response = self.send_request(params).await?;
        handle_response(response).await
    }

    async fn call_api_with_stats(&self, params: Value) -> Result<ResponseWithStats> {
        let response = self.send_request(&params).await?;
        handle_response_with_stats(response).await
    }
}

fn prepare_messages(system_msg: &str, messages: &[Message]) -> Result<Vec<Message>> {
    let mut result = vec![Message {
        role: "system".into(),
        content: system_msg.into(),
    }];
    result.extend_from_slice(messages);
    Ok(result)
}

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

fn build_params(config: &Config, messages: Vec<Message>) -> Value {
    build_params_unified(config, messages, false)
}

fn build_params_stream(config: &Config, messages: &[Message]) -> Result<Value> {
    Ok(build_params_unified(config, messages.to_vec(), true))
}

fn build_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // 添加Bearer认证头，符合OAuth 2.0规范
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
    );

    // 指定请求体内容类型为JSON
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );

    headers
}

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

// 宏：提取响应内容的通用逻辑
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

// 宏：构建RequestStats的通用逻辑
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

async fn handle_response(response: reqwest::Response) -> Result<String> {
    let response = check_response_status(response).await?;
    let completion: CompletionResponse = response.json().await?;
    extract_content!(completion)
}

async fn handle_response_with_stats(response: reqwest::Response) -> Result<ResponseWithStats> {
    let response = check_response_status(response).await?;
    let completion: CompletionResponse = response.json().await?;
    let content = extract_content!(completion)?;
    let stats = build_stats!(completion);
    Ok(ResponseWithStats { content, stats })
}

fn process_stream_chunk(text: &str) -> Result<String> {
    // 检查是否包含SSE数据标记
    if text.contains("data: ") {
        // 逐行处理SSE数据
        for line in text.lines() {
            // 查找有效的数据行：以"data: "开头且不是结束标记
            if line.starts_with("data: ")
                && !line.contains("[DONE]")  // 跳过流结束标记
                && let Ok(json_str) = line.strip_prefix("data: ").ok_or("No data prefix")
                && let Ok(stream_data) = serde_json::from_str::<StreamResponse>(json_str)
                && let Some(content) = stream_data
                    .choices
                    .into_iter()
                    .next()  // 获取第一个选择项
                    .and_then(|c| c.delta.content)
            // 提取增量内容
            {
                return Ok(content);
            }
        }
    }
    // 如果没有找到有效内容，返回空字符串
    Ok(String::new())
}

pub fn message(role: &str, content: &str) -> Message {
    Message {
        role: role.to_string(),
        content: content.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 宏：简化配置字段断言
    macro_rules! assert_config_field {
        ($config:expr, $field:ident, $expected:expr) => {
            assert_eq!($config.$field, $expected);
        };
    }

    // 宏：简化参数断言
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
        assert_config_field!(client.config, model, "tngtech/deepseek-r1t2-chimera:free");
    }
}
