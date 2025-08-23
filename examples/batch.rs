//! # 批量生成示例
//!
//! 这个示例展示了如何使用 NanoAI 库的批量功能：
//! - 从环境变量加载配置
//! - 创建 LLMClient 实例
//! - 并发处理多个提示
//! - 处理结果向量

use nanoai::{batch_generate, LLMClient};
use nanoai::config::Config;
use nanoai::error::Result;

/// 主函数：演示批量文本生成
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
    
    // 定义多个提示
    let prompts = vec![
        "Rust 的所有权系统是什么？",
        "解释一下 Tokio 的异步运行时。",
        "Cargo 是做什么的？",
    ];
    
    // 批量生成响应
    let results = batch_generate(&client, &prompts.iter().map(|s| *s).collect::<Vec<_>>()).await;
    
    // 处理结果
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(response) => println!("提示 {}: {}", i + 1, response),
            Err(e) => eprintln!("提示 {} 错误: {}", i + 1, e),
        }
    }
    
    Ok(())
}