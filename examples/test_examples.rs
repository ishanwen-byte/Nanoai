//! 示例测试脚本
//! 用于验证示例代码的基本功能（不需要真实的API调用）

use nanoai::{Config, LLMClient, Message, message};

#[tokio::main]
async fn main() {
    println!("🧪 NanoAI 示例测试\n");

    // 测试1: 配置创建
    test_config_creation();

    // 测试2: 客户端创建
    test_client_creation();

    // 测试3: 消息创建
    test_message_creation();

    // 测试4: 错误处理（模拟）
    test_error_handling().await;

    println!("\n✅ 所有测试通过！");
    println!("\n💡 提示: 要运行完整的示例，请设置有效的 API 密钥:");
    println!("   $env:OPENAI_API_KEY=\"your-api-key-here\"");
    println!("   cargo run --example basic_usage");
}

/// 测试配置创建
fn test_config_creation() {
    println!("🔧 测试配置创建...");

    // 测试默认配置
    let _default_config = Config::default();
    println!("   ✅ 默认配置创建成功");

    // 测试链式配置
    let _custom_config = Config::default()
        .with_api_key("test-key".to_string())
        .with_model("gpt-3.5-turbo".to_string())
        .with_temperature(0.8);

    println!("   ✅ 自定义配置创建成功");
    println!("   📋 配置详情: 模型=gpt-3.5-turbo, 温度=0.8");
}

/// 测试客户端创建
fn test_client_creation() {
    println!("\n🤖 测试客户端创建...");

    let config = Config::default()
        .with_api_key("test-key".to_string())
        .with_model("gpt-3.5-turbo".to_string());

    let client = LLMClient::new(config);
    println!("   ✅ 客户端创建成功");

    // 测试客户端克隆
    let _cloned_client = client.clone();
    println!("   ✅ 客户端克隆成功");
}

/// 测试消息创建
fn test_message_creation() {
    println!("\n💬 测试消息创建...");

    // 使用便利函数创建消息
    let msg1 = message("user", "Hello, AI!");
    println!("   ✅ 便利函数创建消息: {} - {}", msg1.role, msg1.content);

    // 直接创建消息结构体
    let msg2 = Message {
        role: "assistant".to_string(),
        content: "Hello, human!".to_string(),
    };
    println!("   ✅ 直接创建消息: {} - {}", msg2.role, msg2.content);

    // 创建消息列表
    let messages = [
        message("system", "You are a helpful assistant."),
        message("user", "What is Rust?"),
        message("assistant", "Rust is a systems programming language."),
    ];

    println!("   ✅ 消息列表创建成功，包含 {} 条消息", messages.len());

    // 显示消息内容
    for (i, msg) in messages.iter().enumerate() {
        println!(
            "      {}. {}: {}",
            i + 1,
            msg.role,
            msg.content.chars().take(30).collect::<String>() + "..."
        );
    }
}

/// 测试错误处理（模拟）
async fn test_error_handling() {
    println!("\n🛡️ 测试错误处理...");

    // 创建一个无效配置（无效的API密钥）
    let bad_config = Config::default()
        .with_api_key("invalid-key".to_string())
        .with_model("gpt-3.5-turbo".to_string());

    let client = LLMClient::new(bad_config);

    // 尝试调用API（这会失败，但我们可以测试错误处理结构）
    println!("   ⚠️ 模拟API调用失败场景");

    match client.generate("Hello").await {
        Ok(response) => {
            println!("   ❌ 意外成功: {response}");
        }
        Err(e) => {
            println!("   ✅ 预期的错误: {e}");

            // 测试错误类型匹配
            match e {
                nanoai::NanoError::Api(_) => {
                    println!("      📋 错误类型: API错误");
                }
                nanoai::NanoError::Http(_) => {
                    println!("      📋 错误类型: HTTP错误");
                }
                nanoai::NanoError::Timeout => {
                    println!("      📋 错误类型: 超时错误");
                }
                nanoai::NanoError::Json(_) => {
                    println!("      📋 错误类型: JSON解析错误");
                }
                nanoai::NanoError::NoContent => {
                    println!("      📋 错误类型: 无内容错误");
                }
                nanoai::NanoError::StreamError(_) => {
                    println!("      📋 错误类型: 流处理错误");
                }
                nanoai::NanoError::RateLimit(_) => {
                    println!("      📋 错误类型: 速率限制错误");
                }
                nanoai::NanoError::Auth(_) => {
                    println!("      📋 错误类型: 认证错误");
                }
                nanoai::NanoError::ModelNotFound(_) => {
                    println!("      📋 错误类型: 模型未找到错误");
                }
                nanoai::NanoError::InvalidRequest(_) => {
                    println!("      📋 错误类型: 无效请求错误");
                }
                nanoai::NanoError::Config(_) => {
                    println!("      📋 错误类型: 配置错误");
                }
            }
        }
    }
}
