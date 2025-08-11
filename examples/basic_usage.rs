//! NanoAI 基础使用示例
//! 展示如何使用 nanoai 库进行各种 AI 对话操作

use futures::StreamExt;
use nanoai::{Config, LLMClient, Message, Result, message};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    // 从.env文件获取API密钥和配置
    let (api_key, base_url, model) = if let Ok(key) = dotenv::var("OPENROUTER_API_KEY") {
        let model = dotenv::var("OPENROUTER_MODEL").unwrap_or("openai/gpt-3.5-turbo".to_string());
        (key, Some("https://openrouter.ai/api/v1".to_string()), model)
    } else if let Ok(key) = dotenv::var("OPENAI_API_KEY") {
        (key, None, "gpt-3.5-turbo".to_string())
    } else if let Ok(key) = dotenv::var("API_KEY") {
        (key, None, "gpt-3.5-turbo".to_string())
    } else {
        println!("❌ 错误: 未找到API密钥");
        println!("\n请通过以下方式之一设置API密钥:");
        println!("\n方式1: 创建.env文件 (推荐)");
        println!("   OpenAI: OPENAI_API_KEY=your-openai-key");
        println!("   OpenRouter: OPENROUTER_API_KEY=your-openrouter-key");
        println!("              OPENROUTER_MODEL=your-model-name");
        println!("\n方式2: 设置环境变量");
        println!("   Windows PowerShell: $env:OPENAI_API_KEY=\"your-api-key\"");
        println!("   Windows CMD: set OPENAI_API_KEY=your-api-key");
        return Ok(());
    };

    println!("✅ API密钥已设置");
    println!("🔧 使用模型: {model}");

    println!("🚀 NanoAI 基础使用示例\n");

    // 示例1: 基础配置和简单对话
    basic_chat_example(&api_key, &base_url, &model).await?;

    // 示例2: 自定义配置
    custom_config_example(&api_key, &base_url, &model).await?;

    // 示例3: 多轮对话
    multi_turn_conversation(&api_key, &base_url, &model).await?;

    // 示例4: 流式响应
    streaming_example(&api_key, &base_url, &model).await?;

    // 示例5: 错误处理
    error_handling_example(&api_key, &base_url, &model).await?;

    println!("\n✅ 所有示例执行完成！");
    Ok(())
}

/// 示例1: 基础配置和简单对话
async fn basic_chat_example(api_key: &str, base_url: &Option<String>, model: &str) -> Result<()> {
    println!("📝 示例1: 基础对话");

    // 创建默认配置
    let mut config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    if let Some(url) = base_url {
        config = config.with_base_url(url.clone());
    }

    // 创建客户端
    let client = LLMClient::new(config);

    // 简单对话
    let response = client.generate("你好，请简单介绍一下你自己。").await?;
    println!("🤖 AI回复: {response}");

    println!("✅ 基础对话示例完成\n");
    Ok(())
}

/// 示例2: 自定义配置
async fn custom_config_example(
    api_key: &str,
    base_url: &Option<String>,
    model: &str,
) -> Result<()> {
    println!("⚙️ 示例2: 自定义配置");

    // 创建自定义配置
    let mut config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string())
        .with_temperature(0.9); // 更高的创造性

    if let Some(url) = base_url {
        config = config.with_base_url(url.clone());
    }

    let client = LLMClient::new(config);

    // 创造性写作任务
    let prompt = "写一个关于机器人学会做饭的有趣小故事，大约100字。";
    let response = client.generate(prompt).await?;
    println!("🤖 创意故事: {response}");

    println!("✅ 自定义配置示例完成\n");
    Ok(())
}

/// 示例3: 多轮对话
async fn multi_turn_conversation(
    api_key: &str,
    base_url: &Option<String>,
    model: &str,
) -> Result<()> {
    println!("💬 示例3: 多轮对话");

    let mut config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    if let Some(url) = base_url {
        config = config.with_base_url(url.clone());
    }

    let client = LLMClient::new(config);

    // 构建对话历史
    let messages = vec![
        message("user", "我想学习 Rust 编程语言"),
        message(
            "assistant",
            "太好了！Rust 是一门系统编程语言，以内存安全和高性能著称。你想从哪个方面开始学习？",
        ),
        message("user", "请推荐一些适合初学者的学习资源"),
    ];

    let system_message = "你是一个友好的编程导师，专门帮助初学者学习编程。";
    let response = client
        .generate_with_context(system_message, &messages)
        .await?;

    println!("🤖 编程导师回复: {response}");
    println!("✅ 多轮对话示例完成\n");
    Ok(())
}

/// 示例4: 流式响应
async fn streaming_example(api_key: &str, base_url: &Option<String>, model: &str) -> Result<()> {
    println!("🌊 示例4: 流式响应");

    let mut config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    if let Some(url) = base_url {
        config = config.with_base_url(url.clone());
    }

    let client = LLMClient::new(config);

    println!("🤖 AI正在思考并逐步回复...");
    print!("回复: ");

    // 流式生成
    let mut stream = client
        .generate_stream("请解释什么是函数式编程，并给出一个简单的例子。")
        .await?;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{chunk}");
                // 刷新输出缓冲区以实时显示
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("\n❌ 流式响应错误: {e}");
                break;
            }
        }
    }

    println!("\n✅ 流式响应示例完成\n");
    Ok(())
}

/// 示例5: 错误处理
async fn error_handling_example(
    api_key: &str,
    base_url: &Option<String>,
    model: &str,
) -> Result<()> {
    println!("🛡️ 示例5: 错误处理");

    // 故意使用错误的配置来演示错误处理
    let mut config = Config::default()
        .with_api_key("invalid_key".to_string()) // 无效的API密钥
        .with_model(model.to_string());

    if let Some(url) = base_url {
        config = config.with_base_url(url.clone());
    }

    let client = LLMClient::new(config);

    // 尝试调用API并处理错误
    match client.generate("Hello").await {
        Ok(response) => {
            println!("🤖 意外成功: {response}");
        }
        Err(e) => {
            println!("❌ 预期的错误: {e}");

            // 根据错误类型进行不同处理
            match e {
                nanoai::NanoError::Api(msg) => {
                    println!("   这是一个API错误: {msg}");
                }
                nanoai::NanoError::Http(_) => {
                    println!("   这是一个HTTP错误");
                }
                nanoai::NanoError::Timeout => {
                    println!("   请求超时");
                }
                _ => {
                    println!("   其他类型的错误");
                }
            }
        }
    }

    // 现在使用正确的配置
    println!("\n🔧 使用正确的配置重试...");
    let mut correct_config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    if let Some(url) = base_url {
        correct_config = correct_config.with_base_url(url.clone());
    }

    let correct_client = LLMClient::new(correct_config);
    let response = correct_client
        .generate("简单说一句话证明你正常工作。")
        .await?;
    println!("🤖 正常回复: {response}");

    println!("✅ 错误处理示例完成\n");
    Ok(())
}

/// 辅助函数：演示不同的消息创建方式
#[allow(dead_code)]
fn demonstrate_message_creation() {
    // 方式1: 使用便利函数
    let _msg1 = message("user", "Hello");

    // 方式2: 直接创建结构体
    let _msg2 = Message {
        role: "assistant".to_string(),
        content: "Hi there!".to_string(),
    };

    // 方式3: 批量创建
    let _messages = [
        message("system", "You are a helpful assistant."),
        message("user", "What's the weather like?"),
        message(
            "assistant",
            "I don't have access to real-time weather data.",
        ),
    ];
}
