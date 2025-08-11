//! NanoAI - 轻量级AI客户端库（函数式风格）
//! 实现OpenAI兼容的LLM接口

use futures::{Stream, StreamExt};
use log::{debug, error, info, warn};
use reqwest::{
    Client as ReqwestClient,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

// 错误处理 - 函数式风格的错误类型
#[derive(Debug, Error)]
pub enum NanoError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Request timed out")]
    Timeout,
    #[error("No content in response")]
    NoContent,
    #[error("Stream error: {0}")]
    StreamError(String),
}

pub type Result<T> = std::result::Result<T, NanoError>;

// 不可变数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
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
            system_message: "You are a helpful assistant.".into(),
            temperature: 0.7,
            top_p: 1.0,
            max_tokens: 1024,
            timeout: Duration::from_secs(60),
            retries: 3,
            retry_delay: Duration::from_secs(2),
            api_base: "https://api.openrouter.com/v1".into(),
            api_key: "".into(),
            random_seed: None,
        }
    }
}

impl Config {
    // 纯函数：创建新配置（不修改原有配置）
    pub fn with_model(self, model: String) -> Self {
        Self { model, ..self }
    }

    pub fn with_api_key(self, api_key: String) -> Self {
        Self { api_key, ..self }
    }

    pub fn with_temperature(self, temperature: f32) -> Self {
        Self {
            temperature,
            ..self
        }
    }

    pub fn with_base_url(self, api_base: String) -> Self {
        Self { api_base, ..self }
    }

    pub fn with_random_seed(self, seed: u64) -> Self {
        Self {
            random_seed: Some(seed),
            ..self
        }
    }

    pub fn with_random_seed_auto(self) -> Self {
        Self {
            random_seed: Some(fastrand::u64(..)),
            ..self
        }
    }
}

// 响应类型
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct CompletionChoice {
    message: CompletionMessage,
}

#[derive(Debug, Deserialize)]
struct CompletionMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

// 函数式客户端
#[derive(Debug, Clone)]
pub struct LLMClient {
    config: Config,
    http_client: ReqwestClient,
}

impl LLMClient {
    // 纯函数：创建新客户端
    pub fn new(config: Config) -> Self {
        static INITIALIZED: once_cell::sync::Lazy<std::sync::Mutex<Vec<String>>> =
            once_cell::sync::Lazy::new(|| std::sync::Mutex::new(Vec::new()));

        if let Ok(mut models) = INITIALIZED.lock()
            && !models.contains(&config.model)
        {
            info!("Initialized LLM with model: {}", config.model);
            models.push(config.model.clone());
        }

        Self {
            config,
            http_client: ReqwestClient::builder()
                .danger_accept_invalid_certs(false)
                .use_rustls_tls()
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| ReqwestClient::new()),
        }
    }

    // 生成文本 - 纯函数风格
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_context(
            &self.config.system_message,
            &[Message {
                role: "user".into(),
                content: prompt.into(),
            }],
        )
        .await
    }

    // 带上下文生成 - 函数式组合
    pub async fn generate_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<String> {
        // 函数组合：准备消息 -> 构建参数 -> 带重试调用API
        let msgs = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, &msgs, false)?;
        self.call_with_retry(&params).await
    }

    // 流式生成 - 函数式流处理
    pub async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let system_msg = self.config.system_message.clone();
        let messages = vec![
            Message {
                role: "system".into(),
                content: system_msg,
            },
            Message {
                role: "user".into(),
                content: prompt.into(),
            },
        ];

        let params = build_params(&self.config, &messages, true)?;
        let endpoint = format!("{}/chat/completions", self.config.api_base);
        let headers = build_headers(&self.config.api_key);

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&params)
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    NanoError::Timeout
                } else {
                    e.into()
                }
            })?;

        if !response.status().is_success() {
            return Err(NanoError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await?
            )));
        }

        // 函数式流处理管道
        use futures::future;
        Ok(response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => match std::str::from_utf8(&chunk) {
                        Ok(text) => {
                            // 简化处理：直接返回文本块
                            if text.contains("data: ") {
                                let lines: Vec<&str> = text.lines().collect();
                                for line in lines {
                                    if line.starts_with("data: ")
                                        && !line.contains("[DONE]")
                                        && let Ok(json_str) =
                                            line.strip_prefix("data: ").ok_or("No data prefix")
                                        && let Ok(stream_data) =
                                            serde_json::from_str::<StreamResponse>(json_str)
                                        && let Some(content) = stream_data
                                            .choices
                                            .into_iter()
                                            .next()
                                            .and_then(|c| c.delta.content)
                                    {
                                        return Ok(content);
                                    }
                                }
                            }
                            Ok(String::new())
                        }
                        Err(e) => Err(NanoError::StreamError(format!("Invalid UTF-8: {e}"))),
                    },
                    Err(e) => Err(NanoError::StreamError(e.to_string())),
                }
            })
            .filter(|result| {
                future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            }))
    }

    // 带上下文流式生成
    pub async fn generate_stream_with_context(
        &self,
        system_msg: &str,
        messages: &[Message],
    ) -> Result<impl Stream<Item = Result<String>> + '_> {
        let msgs = prepare_messages(system_msg, messages)?;
        let params = build_params(&self.config, &msgs, true)?;
        let endpoint = format!("{}/chat/completions", self.config.api_base);
        let headers = build_headers(&self.config.api_key);

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&params)
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    NanoError::Timeout
                } else {
                    e.into()
                }
            })?;

        if !response.status().is_success() {
            return Err(NanoError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await?
            )));
        }

        // 函数式流处理管道
        use futures::future;
        Ok(response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => match std::str::from_utf8(&chunk) {
                        Ok(text) => {
                            // 简化处理：直接返回文本块
                            if text.contains("data: ") {
                                let lines: Vec<&str> = text.lines().collect();
                                for line in lines {
                                    if line.starts_with("data: ")
                                        && !line.contains("[DONE]")
                                        && let Ok(json_str) =
                                            line.strip_prefix("data: ").ok_or("No data prefix")
                                        && let Ok(stream_data) =
                                            serde_json::from_str::<StreamResponse>(json_str)
                                        && let Some(content) = stream_data
                                            .choices
                                            .into_iter()
                                            .next()
                                            .and_then(|c| c.delta.content)
                                    {
                                        return Ok(content);
                                    }
                                }
                            }
                            Ok(String::new())
                        }
                        Err(e) => Err(NanoError::StreamError(format!("Invalid UTF-8: {e}"))),
                    },
                    Err(e) => Err(NanoError::StreamError(e.to_string())),
                }
            })
            .filter(|result| {
                future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            }))
    }

    // 带重试的API调用 - 函数式递归实现
    async fn call_with_retry(&self, params: &Value) -> Result<String> {
        let mut retries_left = self.config.retries;
        loop {
            match self.call_api(params).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retries_left > 0 {
                        warn!("Request failed: {e}. Retrying...");
                        sleep(self.config.retry_delay).await;
                        retries_left -= 1;
                    } else {
                        error!("All retries exhausted: {e}");
                        return Err(e);
                    }
                }
            }
        }
    }

    // 实际API调用
    async fn call_api(&self, params: &Value) -> Result<String> {
        debug!("API parameters: {params:?}");

        let endpoint = format!("{}/chat/completions", self.config.api_base);
        let headers = build_headers(&self.config.api_key);

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
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
            })?;

        handle_response(response).await
    }
}

// 纯函数：准备消息列表
fn prepare_messages(system_msg: &str, messages: &[Message]) -> Result<Vec<Message>> {
    let mut result = vec![Message {
        role: "system".into(),
        content: system_msg.into(),
    }];
    result.extend_from_slice(messages);
    Ok(result)
}

// 纯函数：构建请求参数
fn build_params(config: &Config, messages: &[Message], stream: bool) -> Result<Value> {
    let mut params = serde_json::json!({
        "model": config.model,
        "messages": messages,
        "stream": stream,
    });

    // 根据模型调整参数
    if config.api_base == "https://api.openrouter.com/v1" && config.model.starts_with('o') {
        params["max_completion_tokens"] = config.max_tokens.into();
    } else {
        params["temperature"] = config.temperature.into();
        params["top_p"] = config.top_p.into();
        params["max_tokens"] = config.max_tokens.into();
    }

    // 添加随机种子（使用fastrand）
    if let Some(seed) = config.random_seed {
        params["seed"] = seed.into();
    }

    Ok(params)
}

// 纯函数：构建请求头
fn build_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    headers
}

// 纯函数：处理API响应
async fn handle_response(response: reqwest::Response) -> Result<String> {
    if !response.status().is_success() {
        return Err(NanoError::Api(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await?
        )));
    }

    let completion: CompletionResponse = response.json().await?;
    completion
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or(NanoError::NoContent)
}

// 工具函数：创建消息
pub fn message(role: &str, content: &str) -> Message {
    Message {
        role: role.into(),
        content: content.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.model, "tngtech/deepseek-r1t2-chimera:free");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.random_seed, None);
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = Config::default()
            .with_model("gpt-4".to_string())
            .with_api_key("test-key".to_string())
            .with_temperature(0.5)
            .with_random_seed(12345);
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.random_seed, Some(12345));
    }

    #[test]
    fn test_config_random_seed_auto() {
        let config1 = Config::default().with_random_seed_auto();
        let config2 = Config::default().with_random_seed_auto();
        assert!(config1.random_seed.is_some());
        assert!(config2.random_seed.is_some());
        // 随机种子应该不同（概率极高）
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
        let params = build_params(&config, &messages, false).unwrap();
        assert_eq!(params["model"], "tngtech/deepseek-r1t2-chimera:free");
        assert_eq!(params["stream"], false);
        // 使用近似比较浮点数
        let temp = params["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001);
        assert!(params.get("seed").is_none());
    }

    #[test]
    fn test_build_params_with_seed() {
        let config = Config::default().with_random_seed(42);
        let messages = vec![message("user", "test")];

        let params = build_params(&config, &messages, false).unwrap();
        
        assert_eq!(params["seed"], 42);
    }

    #[test]
    fn test_build_params_openrouter() {
        let config = Config::default()
            .with_base_url("https://api.openrouter.com/v1".to_string())
            .with_model("openai/gpt-4".to_string());
        let messages = vec![message("user", "test")];
        let params = build_params(&config, &messages, false).unwrap();
        assert_eq!(params["max_completion_tokens"], 1024);
        assert!(params.get("max_tokens").is_none());
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
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.model, "tngtech/deepseek-r1t2-chimera:free");
    }
}