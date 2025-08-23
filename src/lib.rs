//! # NanoAI - 轻量级 LLM 客户端库
//!
//! NanoAI 是一个专为大语言模型 API 设计的轻量级 Rust 客户端库，
//! 提供简洁、函数式的接口来与各种大语言模型进行交互。
//!
//! ## 主要特性
//!
//! - 🚀 **异步支持**：基于 `tokio` 的完全异步实现，性能卓越。
//! - 🔄 **流式响应**：支持实时流式文本生成，提供即时反馈。
//! - 📊 **统计信息**：可选的详细请求统计和性能监控。
//! - 🔧 **灵活配置**：支持环境变量和 Builder 模式，轻松定制客户端。
//! - 🛡️ **错误处理**：完善的错误类型和基于 `backoff` 的自动重试机制。
//! - 🎯 **函数式设计**：遵循 Rust 函数式编程最佳实践，代码简洁、可预测。
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use nanoai::client::LLMClient;
//! use nanoai::config::Config;
//! use nanoai::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 从环境变量加载配置 (需要设置 YOPENROUTER_API_KEY)
//!     let config = Config::from_env()?;
//!     let client = LLMClient::new(config);
//!     
//!     // 发起请求并获取响应
//!     let response = client.generate("你好，世界！").await?;
//!     println!("模型响应: {}", response);
//!     
//!     Ok(())
//! }
//! ```

// 模块定义
pub mod client;
pub mod config;
pub mod error;
pub mod stream;
pub mod types;
pub mod utils;

pub use client::LLMClient;
use error::Result;
use futures::future::join_all;
use types::ResponseWithStats;

// ================================================================================================
//  并发工具函数
// ================================================================================================

/// 批量生成文本响应
///
/// 并发处理多个提示，返回所有结果的向量。
/// 成功的结果包含生成的文本，失败的结果包含错误信息。
///
/// # 参数
///
/// * `client` - `LLMClient` 实例的引用。
/// * `prompts` - 一个字符串切片，包含所有需要处理的提示。
///
/// # 返回
///
/// 一个向量，包含每个提示的处理结果 (`Result<String>`)，顺序与输入一致。
///
/// # 示例
///
/// ```rust,no_run
/// use nanoai::client::LLMClient;
/// use nanoai::config::Config;
/// use nanoai::batch_generate;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Config::from_env()?;
///     let client = LLMClient::new(config);
///     
///     let prompts = vec![
///         "请解释什么是人工智能?",
///         "Rust 编程语言有什么优势?",
///     ];
///     
///     let results = batch_generate(&client, &prompts).await;
///     
///     for result in results {
///         match result {
///             Ok(response) => println!("成功: {}", response),
///             Err(e) => eprintln!("失败: {}", e),
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn batch_generate(client: &LLMClient, prompts: &[&str]) -> Vec<Result<String>> {
    let futures = prompts.iter().map(|p| client.generate(p)).collect::<Vec<_>>();
    join_all(futures).await
}

/// 批量生成文本响应（带统计信息）
///
/// 并发处理多个提示并返回详细的统计信息。
///
/// # 参数
///
/// * `client` - `LLMClient` 实例的引用。
/// * `prompts` - 一个字符串切片，包含所有需要处理的提示。
///
/// # 返回
///
/// 一个向量，包含每个提示的处理结果 (`Result<ResponseWithStats>`)，顺序与输入一致。
pub async fn batch_generate_with_stats(
    client: &LLMClient,
    prompts: &[&str],
) -> Vec<Result<ResponseWithStats>> {
    let futures = prompts
        .iter()
        .map(|p| client.generate_with_stats(p))
        .collect::<Vec<_>>();
    join_all(futures).await
}
