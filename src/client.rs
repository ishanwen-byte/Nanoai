//! LLM 客户端核心模块
use crate::{
    config::Config,
    error::{NanoError, Result},
    stream::StreamWrapper,
    types::{CompletionResponse, Message, RequestStats, ResponseWithStats, Role, StreamCompletionResponse},
    utils::{message, prepare_messages},
};
use futures::{Stream, StreamExt};
use log::error;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client, RequestBuilder, Response,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

// ================================================================================================
// 核心客户端模块
// ================================================================================================

/// LLM 客户端
///
/// 提供与 OpenRouter API 交互的核心功能，支持同步和流式请求
#[derive(Debug, Clone)]
pub struct LLMClient {
    client: Arc<Client>,
    config: Arc<Config>,
    semaphore: Arc<Semaphore>,
    stream_handler: StreamWrapper,
}

impl LLMClient {
    /// 创建一个新的 `LLMClient` 实例
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .tcp_keepalive(config.tcp_keepalive)
            .tcp_nodelay(config.tcp_nodelay)
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|e| {
                error!("Failed to build reqwest client: {}", e);
                Client::new()
            });

        let semaphore = Semaphore::new(config.max_concurrent_requests.unwrap_or(64));

        Self {
            client: Arc::new(client),
            config: Arc::new(config),
            semaphore: Arc::new(semaphore),
            stream_handler: StreamWrapper::new(),
        }
    }

    /// 构建 API 请求所需的 HTTP 标头
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .map_err(|e| NanoError::InvalidRequest(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// 使用重试逻辑发送 HTTP 请求
    async fn call_api_with_retry(&self, request_builder: RequestBuilder) -> Result<Response> {
        // Note: backoff crate is not used here to simplify, add it back if needed.
        let permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| NanoError::Api(format!("Semaphore acquisition failed: {}", e)))?;

        let response_result = request_builder.send().await;
        drop(permit);

        let response = response_result?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let error_msg = format!("Request failed with status: {}", response.status());
            Err(NanoError::Api(error_msg))
        }
    }

    /// 调用 API 并返回带统计信息的完整响应
    async fn call_api_with_stats(&self, params: &Value) -> Result<ResponseWithStats> {
        let endpoint = format!("{}/chat/completions", self.config.api_base);
        let headers = self.build_headers()?;
        let request_builder = self.client.post(&endpoint).headers(headers).json(params);

        let response = self.call_api_with_retry(request_builder).await?;
        let completion = response.json::<CompletionResponse>().await?;
        let content = completion
            .choices
            .first()
            .map_or(String::new(), |c| c.message.content.clone());

        let u = completion.usage;
        let mut stats = RequestStats {
            prompt_tokens: Some(u.prompt_tokens),
            completion_tokens: Some(u.completion_tokens),
            total_tokens: Some(u.total_tokens),
            ..RequestStats::default()
        };
        stats.model = self.config.model.clone();
        stats.timestamp = Some(std::time::SystemTime::now());

        Ok(ResponseWithStats { content, stats })
    }

    /// 内部辅助函数，用于生成响应，处理上下文和统计信息
    async fn generate_internal(
        &self,
        system_msg: Option<&str>,
        messages: &[Message],
    ) -> Result<ResponseWithStats> {
        let start_time = Instant::now();
        let system_message = system_msg.unwrap_or(&self.config.system_message);
        let prepared_messages = prepare_messages(system_message, messages);

        let params = serde_json::json!({
            "model": &self.config.model,
            "messages": prepared_messages,
            "temperature": &self.config.temperature,
            "top_p": &self.config.top_p,
            "max_tokens": &self.config.max_tokens,
            "stream": false,
        });

        let mut response = self.call_api_with_stats(&params).await?;
        let duration = start_time.elapsed();
        response.stats.duration_ms = duration.as_millis() as u64;
        Ok(response)
    }

    /// 为给定的提示生成响应
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_stats(prompt)
            .await
            .map(|res| res.content)
    }

    /// 为给定的提示生成响应，包括性能统计信息
    pub async fn generate_with_stats(&self, prompt: &str) -> Result<ResponseWithStats> {
        let messages = vec![message(Role::User, prompt)];
        self.generate_internal(None, &messages).await
    }

    /// 为给定的消息列表生成响应
    pub async fn batch_generate(&self, messages: &[Message]) -> Result<String> {
        self.batch_generate_with_stats(messages)
            .await
            .map(|res| res.content)
    }

    /// 为给定的消息列表生成响应，包括性能统计信息
    pub async fn batch_generate_with_stats(
        &self,
        messages: &[Message],
    ) -> Result<ResponseWithStats> {
        self.generate_internal(None, messages).await
    }

    /// 为给定的提示生成流式响应
    pub async fn stream_generate(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>>> {
        let messages = vec![message(Role::User, prompt)];
        self.stream_internal(messages).await
    }

    /// 为给定的消息列表生成流式响应
    pub async fn stream_batch_generate(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<String>>> {
        self.stream_internal(messages).await
    }

    /// 内部辅助函数，用于处理流式响应
    async fn stream_internal(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<String>>> {
        let endpoint = format!("{}/chat/completions", self.config.api_base);
        let mut headers = self.build_headers()?;
        headers.insert("Accept", HeaderValue::from_static("text/event-stream"));

        let system_message = &self.config.system_message;
        let prepared_messages = prepare_messages(system_message, &messages);

        let params = serde_json::json!({
            "model": &self.config.model,
            "messages": prepared_messages,
            "temperature": &self.config.temperature,
            "top_p": &self.config.top_p,
            "max_tokens": &self.config.max_tokens,
            "stream": true,
        });

        let request_builder = self.client.post(&endpoint).headers(headers).json(&params);
        let response = self.call_api_with_retry(request_builder).await?;

        let stream = self
            .stream_handler
            .stream(response.bytes_stream());
        Ok(stream.map(|res: Result<StreamCompletionResponse>| {
            res.map(|chunk| {
                let content = chunk.choices.first().and_then(|c| c.delta.content.as_ref());
                content.cloned().unwrap_or_default()
            })
        }).boxed())
    }
}
