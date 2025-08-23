//! # 流式响应示例
//!
//! 这个示例展示了如何使用 NanoAI 库的流式功能：
//! - 从环境变量加载配置
//! - 创建 LLMClient 实例
//! - 发送流式生成请求
//! - 实时处理和输出响应块

use nanoai::client::LLMClient;
use nanoai::config::Config;
use nanoai::error::Result;
use futures::StreamExt;

/// 主函数：演示流式文本生成
///
/// # 返回
///
/// 返回 `Result<()>`，成功时为空，失败时包含错误
#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量加载配置
    let config = Config::from_env()?;
    
    // 创建客户端
    let client = LLMClient::new(config);
    
    // 定义提示
    let prompt = "请写一段 9000字的 母爱的作文。";
    
    // 生成流式响应
    let mut stream = client.stream_generate(prompt).await?;
    
    // 实时处理流
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => print!("{}", chunk),
            Err(e) => eprintln!("错误: {}", e),
        }
    }
    
    println!();
    
    Ok(())
}