//! NanoAI 流式处理示例
//! 专门展示流式响应的各种使用场景

use futures::StreamExt;
use nanoai::{Config, LLMClient, Result, message};
use std::io::{self, Write};
use std::time::Instant;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    // 从.env文件获取API密钥和配置
    let (api_key, model) = if let Ok(key) = dotenvy::var("OPENROUTER_API_KEY") {
        let model = dotenvy::var("OPENROUTER_MODEL")
            .unwrap_or("tngtech/deepseek-r1t2-chimera:free".to_string());
        (key, model)
    } else if let Ok(key) = dotenvy::var("API_KEY") {
        (key, "tngtech/deepseek-r1t2-chimera:free".to_string())
    } else {
        println!("❌ 错误: 未找到OpenRouter API密钥");
        println!("\n请通过以下方式之一设置API密钥:");
        println!("\n方式1: 创建.env文件 (推荐)");
        println!("   OPENROUTER_API_KEY=your-openrouter-key");
        println!("   OPENROUTER_MODEL=your-model-name (可选)");
        println!("\n方式2: 设置环境变量");
        println!("   Windows PowerShell: $env:OPENROUTER_API_KEY=\"your-api-key\"");
        println!("   Windows CMD: set OPENROUTER_API_KEY=your-api-key");
        return Ok(());
    };

    println!("✅ API密钥已设置");
    println!("🔧 使用模型: {model}");

    println!("🌊 NanoAI 流式处理示例\n");

    // 示例1: 基础流式响应
    basic_streaming_example(&api_key, &model).await?;

    // 示例2: 实时打字效果
    typewriter_effect_example(&api_key, &model).await?;

    // 示例3: 流式对话
    streaming_conversation_example(&api_key, &model).await?;

    // 示例4: 流式内容处理
    stream_processing_example(&api_key, &model).await?;

    // 示例5: 流式错误处理
    streaming_error_handling(&api_key, &model).await?;

    println!("\n✅ 所有流式处理示例执行完成！");
    Ok(())
}

/// 示例1: 基础流式响应
async fn basic_streaming_example(api_key: &str, model: &str) -> Result<()> {
    println!("🌊 示例1: 基础流式响应");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    let client = LLMClient::new(config);

    println!("🤖 AI正在生成回答...");
    println!("回答: ");

    let mut stream = client
        .generate_stream("请详细解释什么是Rust编程语言的所有权系统。")
        .await?;

    let mut full_response = String::new();
    let start_time = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{chunk}");
                io::stdout().flush().unwrap();
                full_response.push_str(&chunk);
            }
            Err(e) => {
                eprintln!("\n❌ 流式响应错误: {e}");
                break;
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n\n📊 统计信息:");
    println!("   总字符数: {}", full_response.len());
    println!("   总耗时: {:?}", elapsed);
    println!(
        "   平均速度: {:.1} 字符/秒",
        full_response.len() as f64 / elapsed.as_secs_f64()
    );

    println!("✅ 基础流式响应示例完成\n");
    Ok(())
}

/// 示例2: 实时打字效果
async fn typewriter_effect_example(api_key: &str, model: &str) -> Result<()> {
    println!("⌨️ 示例2: 实时打字效果");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string())
        .with_temperature(0.8);

    let client = LLMClient::new(config);

    println!("🤖 AI正在创作一首关于编程的诗...");
    println!("\n📝 诗歌:");
    println!("─────────────────────────────────");

    let mut stream = client
        .generate_stream("请写一首关于程序员生活的现代诗，要有节奏感和韵律。")
        .await?;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // 模拟打字机效果 - 逐字符显示
                for ch in chunk.chars() {
                    print!("{}", ch);
                    io::stdout().flush().unwrap();

                    // 根据字符类型调整延迟
                    let delay = match ch {
                        '。' | '！' | '？' => 200, // 句号后稍长停顿
                        '，' | '；' => 100,        // 逗号后短停顿
                        ' ' => 50,                 // 空格后很短停顿
                        _ => 30,                   // 普通字符
                    };

                    sleep(Duration::from_millis(delay)).await;
                }
            }
            Err(e) => {
                eprintln!("\n❌ 流式响应错误: {e}");
                break;
            }
        }
    }

    println!("\n─────────────────────────────────");
    println!("✅ 打字效果示例完成\n");
    Ok(())
}

/// 示例3: 流式对话
async fn streaming_conversation_example(api_key: &str, model: &str) -> Result<()> {
    println!("💬 示例3: 流式对话");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    let client = LLMClient::new(config);

    let system_message = "你是一个友好的AI助手，喜欢用表情符号，回答要简洁有趣。";

    // 模拟多轮对话
    let conversations = [
        "你好！今天天气怎么样？",
        "我想学习一门新的编程语言，有什么推荐吗？",
        "Rust语言有什么特点？",
    ];

    let mut message_history = Vec::new();

    for (i, user_input) in conversations.iter().enumerate() {
        println!("\n🔄 对话轮次 {}", i + 1);
        println!("👤 用户: {user_input}");

        // 添加用户消息到历史
        message_history.push(message("user", user_input));

        print!("🤖 AI: ");
        io::stdout().flush().unwrap();

        // 流式生成回复
        let mut stream = client
            .generate_stream_with_context(system_message, &message_history)
            .await?;

        let mut ai_response = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    print!("{}", chunk);
                    io::stdout().flush().unwrap();
                    ai_response.push_str(&chunk);
                }
                Err(e) => {
                    eprintln!("\n❌ 流式响应错误: {e}");
                    break;
                }
            }
        }

        // 添加AI回复到历史
        message_history.push(message("assistant", &ai_response));

        println!(); // 换行
    }

    println!("✅ 流式对话示例完成\n");
    Ok(())
}

/// 示例4: 流式内容处理
async fn stream_processing_example(api_key: &str, model: &str) -> Result<()> {
    println!("🔄 示例4: 流式内容处理");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    let client = LLMClient::new(config);

    println!("🤖 AI正在生成技术文章...");

    let mut stream = client
        .generate_stream("请写一篇关于'如何优化Rust程序性能'的技术文章，包含具体的代码示例。")
        .await?;

    let mut word_count = 0;
    let mut sentence_count = 0;
    let mut paragraph_count = 0;
    let mut current_word = String::new();
    let mut buffer = String::new();

    println!("\n📄 文章内容:");
    println!("═══════════════════════════════════════");

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush().unwrap();

                // 实时统计
                for ch in chunk.chars() {
                    buffer.push(ch);

                    match ch {
                        ' ' | '\n' | '\t' => {
                            if !current_word.is_empty() {
                                word_count += 1;
                                current_word.clear();
                            }
                        }
                        '。' | '！' | '？' => {
                            sentence_count += 1;
                        }
                        _ => {
                            current_word.push(ch);
                        }
                    }
                }

                // 检测段落
                if chunk.contains("\n\n") {
                    paragraph_count += chunk.matches("\n\n").count();
                }
            }
            Err(e) => {
                eprintln!("\n❌ 流式响应错误: {e}");
                break;
            }
        }
    }

    // 处理最后一个词
    if !current_word.is_empty() {
        word_count += 1;
    }

    println!("\n═══════════════════════════════════════");
    println!("📊 实时统计结果:");
    println!("   字符数: {}", buffer.len());
    println!("   词数: {word_count}");
    println!("   句子数: {}", sentence_count);
    println!("   段落数: {}", paragraph_count.max(1));

    println!("✅ 流式内容处理示例完成\n");
    Ok(())
}

/// 示例5: 流式错误处理
async fn streaming_error_handling(api_key: &str, model: &str) -> Result<()> {
    println!("🛡️ 示例5: 流式错误处理");

    // 首先演示正常的流式处理
    println!("🔄 正常流式处理:");
    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string());

    let client = LLMClient::new(config);

    let mut stream = client.generate_stream("简单介绍一下Rust语言。").await?;

    let mut chunk_count = 0;
    let mut error_count = 0;

    print!("🤖 回答: ");

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                print!("{}", chunk);
                io::stdout().flush().unwrap();
                chunk_count += 1;
            }
            Err(e) => {
                error_count += 1;
                eprintln!("\n⚠️ 处理第 {} 个数据块时出错: {}", chunk_count + 1, e);

                // 根据错误类型决定是否继续
                match e {
                    nanoai::NanoError::StreamError(_) => {
                        println!("🔄 尝试继续处理下一个数据块...");
                        continue;
                    }
                    _ => {
                        println!("❌ 严重错误，停止处理");
                        break;
                    }
                }
            }
        }
    }

    println!("\n\n📊 处理统计:");
    println!("   成功处理的数据块: {}", chunk_count);
    println!("   错误数量: {}", error_count);

    // 演示错误配置的处理
    println!("\n🔄 错误配置演示:");
    let bad_config = Config::default()
        .with_api_key("invalid_key".to_string())
        .with_model(model.to_string());

    let bad_client = LLMClient::new(bad_config);

    match bad_client.generate_stream("Hello").await {
        Ok(mut stream) => {
            println!("🔄 开始处理流...");
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        print!("{chunk}");
                    }
                    Err(e) => {
                        println!("❌ 预期的流式错误: {e}");
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ 预期的初始化错误: {e}");
        }
    }

    println!("✅ 流式错误处理示例完成\n");
    Ok(())
}
