//! NanoAI 高级使用示例
//! 展示更复杂的使用场景，包括并发处理、批量操作等

use futures::{StreamExt, stream};
use nanoai::{Config, LLMClient, Result, message};
use std::env;
use std::time::{Duration, Instant};
use tokio;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    let api_key = env::var("OPENAI_API_KEY")
        .or_else(|_| env::var("API_KEY"))
        .expect("请设置 OPENAI_API_KEY 或 API_KEY 环境变量");

    println!("🚀 NanoAI 高级使用示例\n");

    // 示例1: 并发处理多个请求
    concurrent_requests_example(&api_key).await?;

    // 示例2: 批量文本处理
    batch_processing_example(&api_key).await?;

    // 示例3: 智能对话系统
    intelligent_chat_system(&api_key).await?;

    // 示例4: 性能测试和监控
    performance_monitoring_example(&api_key).await?;

    // 示例5: 不同模型比较
    model_comparison_example(&api_key).await?;

    println!("\n✅ 所有高级示例执行完成！");
    Ok(())
}

/// 示例1: 并发处理多个请求
async fn concurrent_requests_example(api_key: &str) -> Result<()> {
    println!("⚡ 示例1: 并发处理多个请求");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model("gpt-3.5-turbo".to_string());

    let client = LLMClient::new(config);

    // 准备多个不同的问题
    let questions = vec![
        "什么是人工智能？",
        "解释一下机器学习的基本概念",
        "深度学习和传统机器学习有什么区别？",
        "什么是神经网络？",
        "自然语言处理的主要应用有哪些？",
    ];

    let start_time = Instant::now();

    // 并发执行所有请求
    let tasks: Vec<_> = questions
        .into_iter()
        .enumerate()
        .map(|(i, question)| {
            let client = client.clone();
            let question = question.to_string();
            tokio::spawn(async move {
                println!("🔄 开始处理问题 {}: {question}", i + 1);
                let result = client.generate(&question).await;
                (i + 1, question, result)
            })
        })
        .collect();

    // 等待所有任务完成
    for task in tasks {
        match task.await {
            Ok((index, question, result)) => match result {
                Ok(answer) => {
                    println!("✅ 问题 {index}: {question}");
                    println!(
                        "🤖 回答: {}\n",
                        answer.chars().take(100).collect::<String>() + "..."
                    );
                }
                Err(e) => {
                    println!("❌ 问题 {index} 处理失败: {e}\n");
                }
            },
            Err(e) => {
                println!("❌ 任务执行失败: {e}\n");
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("⏱️ 并发处理耗时: {elapsed:?}");
    println!("✅ 并发请求示例完成\n");
    Ok(())
}

/// 示例2: 批量文本处理
async fn batch_processing_example(api_key: &str) -> Result<()> {
    println!("📦 示例2: 批量文本处理");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model("gpt-3.5-turbo".to_string())
        .with_temperature(0.3); // 更一致的输出

    let client = LLMClient::new(config);

    // 模拟需要处理的文本列表
    let texts = vec![
        "今天天气真好，阳光明媚。",
        "我喜欢在周末读书和看电影。",
        "编程是一门艺术，也是一门科学。",
        "人工智能正在改变我们的世界。",
        "学习新技能需要耐心和坚持。",
    ];

    println!("🔄 开始批量情感分析...");

    // 使用流式处理批量文本
    let results: Vec<_> = stream::iter(texts)
        .map(|text| {
            let client = client.clone();
            async move {
                let prompt = format!(
                    "请分析以下文本的情感倾向（积极/消极/中性），并给出简短解释：\n\n\"{text}\""
                );
                let result = client.generate(&prompt).await;
                (text, result)
            }
        })
        .buffer_unordered(3) // 限制并发数为3
        .collect()
        .await;

    // 显示结果
    for (text, result) in results {
        match result {
            Ok(analysis) => {
                println!("📝 文本: {text}");
                println!("🎭 分析: {analysis}\n");
            }
            Err(e) => {
                println!("❌ 处理失败 '{text}': {e}\n");
            }
        }
    }

    println!("✅ 批量处理示例完成\n");
    Ok(())
}

/// 示例3: 智能对话系统
async fn intelligent_chat_system(api_key: &str) -> Result<()> {
    println!("🧠 示例3: 智能对话系统");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model("gpt-3.5-turbo".to_string());

    let client = LLMClient::new(config);

    // 模拟一个智能客服对话
    let system_prompt =
        "你是一个专业的技术支持客服，友好、耐心、专业。你需要帮助用户解决技术问题。";

    // 模拟对话历史
    let mut conversation = vec![message("user", "你好，我的电脑启动很慢，怎么办？")];

    println!("💬 开始智能对话...");

    for turn in 1..=3 {
        println!("\n🔄 对话轮次 {turn}");

        // 获取AI回复
        let response = client
            .generate_with_context(system_prompt, &conversation)
            .await?;

        println!("🤖 客服: {response}");

        // 添加AI回复到对话历史
        conversation.push(message("assistant", &response));

        // 模拟用户的后续问题
        let user_followup = match turn {
            1 => "我已经清理了磁盘，还有其他建议吗？",
            2 => "好的，我会检查启动项。还有什么需要注意的吗？",
            _ => "谢谢你的帮助！",
        };

        println!("👤 用户: {user_followup}");
        conversation.push(message("user", user_followup));
    }

    println!("\n✅ 智能对话系统示例完成\n");
    Ok(())
}

/// 示例4: 性能测试和监控
async fn performance_monitoring_example(api_key: &str) -> Result<()> {
    println!("📊 示例4: 性能测试和监控");

    let config = Config::default()
        .with_api_key(api_key.to_string())
        .with_model("gpt-3.5-turbo".to_string());

    let client = LLMClient::new(config);

    let test_prompt = "请用一句话描述人工智能。";
    let test_count = 5;

    println!("🔄 执行 {test_count} 次性能测试...");

    let mut response_times = Vec::new();
    let mut success_count = 0;

    for i in 1..=test_count {
        let start = Instant::now();

        match client.generate(test_prompt).await {
            Ok(response) => {
                let elapsed = start.elapsed();
                response_times.push(elapsed);
                success_count += 1;

                println!(
                    "✅ 测试 {}/{}: 耗时 {:?}, 响应长度: {} 字符",
                    i,
                    test_count,
                    elapsed,
                    response.len()
                );
            }
            Err(e) => {
                println!("❌ 测试 {i}/{test_count} 失败: {e}");
            }
        }

        // 避免请求过于频繁
        if i < test_count {
            sleep(Duration::from_millis(500)).await;
        }
    }

    // 计算统计信息
    if !response_times.is_empty() {
        let total_time: Duration = response_times.iter().sum();
        let avg_time = total_time / response_times.len() as u32;
        let min_time = response_times.iter().min().unwrap();
        let max_time = response_times.iter().max().unwrap();

        println!("\n📈 性能统计:");
        println!(
            "   成功率: {}/{} ({:.1}%)",
            success_count,
            test_count,
            (success_count as f64 / test_count as f64) * 100.0
        );
        println!("   平均响应时间: {avg_time:?}");
        println!("   最快响应时间: {min_time:?}");
        println!("   最慢响应时间: {max_time:?}");
    }

    println!("✅ 性能监控示例完成\n");
    Ok(())
}

/// 示例5: 不同模型比较
async fn model_comparison_example(api_key: &str) -> Result<()> {
    println!("🔬 示例5: 不同模型比较");

    let models = vec!["gpt-3.5-turbo", "gpt-4o-mini"];
    let test_prompt = "请用创意的方式解释什么是递归。";

    println!("🔄 使用不同模型生成回答...");

    for model in models {
        println!("\n🤖 模型: {model}");

        let config = Config::default()
            .with_api_key(api_key.to_string())
            .with_model(model.to_string())
            .with_temperature(0.8);

        let client = LLMClient::new(config);
        let start = Instant::now();

        match client.generate(test_prompt).await {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!("⏱️ 响应时间: {elapsed:?}");
                println!("📝 回答: {response}");
            }
            Err(e) => {
                println!("❌ 模型 {model} 调用失败: {e}");
            }
        }
    }

    println!("\n✅ 模型比较示例完成\n");
    Ok(())
}

/// 辅助函数：创建测试配置
#[allow(dead_code)]
fn create_test_config(api_key: &str, model: &str) -> Config {
    Config::default()
        .with_api_key(api_key.to_string())
        .with_model(model.to_string())
        .with_temperature(0.7)
}

/// 辅助函数：格式化响应时间
#[allow(dead_code)]
fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    }
}
