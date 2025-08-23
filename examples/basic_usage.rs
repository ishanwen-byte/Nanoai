//! # 基本使用示例
//!
//! 这个示例展示了如何使用 NanoAI 库的基本功能：
//! - 从环境变量加载配置
//! - 创建 LLMClient 实例
//! - 发送简单生成请求
//! - 处理响应

use nanoai::client::LLMClient;
use nanoai::config::Config;
use nanoai::error::Result;

/// 主函数：演示基本文本生成
///
/// # 返回
///
/// 返回 `Result<()>`，成功时为空，失败时包含错误
#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量加载配置
    let config = Config::from_env()?;
    
    // 创建客户端
    let client = LLMClient::new(config.clone());
    
    println!("模型: {}", config.model());
    println!("API 密钥: {}", config.api_key());    
    println!("基础 URL: {}", config.api_base());

    // 定义提示
    let prompt = "请用一句话解释 Rust 语言的优势。";
    println!("提示: {}", &prompt);
    
    // 生成响应
    let response = client.generate(prompt).await?;
    
    // 输出结果
    println!("模型响应: {}", response);
    
    Ok(())
}