//! 并发聊天示例
//! 
//! 这个示例展示如何同时发起多个聊天请求，并异步处理响应。
//! 适用于需要批量处理多个问题或对话的场景。

use nanoai::{Config, LLMClient};
use tokio;
use futures::future::join_all;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NanoAI 并发聊天示例\n");

    // 步骤1: 从环境变量获取API密钥
    dotenvy::dotenv().ok();
    
    let (api_key, model) = if let Ok(key) = dotenvy::var("OPENROUTER_API_KEY") {
        let model = dotenvy::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "tngtech/deepseek-r1t2-chimera:free".to_string());
        println!("🌐 使用 OpenRouter 配置");
        (key, model)
    } else if let Ok(key) = dotenvy::var("API_KEY") {
        println!("🌐 使用 OpenRouter 配置 (通用API密钥)");
        (key, "tngtech/deepseek-r1t2-chimera:free".to_string())
    } else {
        eprintln!("❌ 错误: 请在 .env 文件中设置 OPENROUTER_API_KEY 或 API_KEY");
        std::process::exit(1);
    };

    println!("✅ API密钥已设置");
    // println!("🔧 使用模型: {}", model);
    
    // 步骤2: 创建配置和客户端
    let config = Config::default()
        .with_api_key(api_key)
        .with_model(model)
        .with_max_tokens(32000)
        .with_temperature(0.7)
        .with_random_seed_auto(); // 每个请求使用不同的随机种子

    let client = LLMClient::new(config);
    println!("🤖 创建AI客户端...\n");

    // 步骤3: 定义三个不同的问题
    let questions = vec![
        ("问题1", "请用一句话解释什么是人工智能？"),
        ("问题2", "请推荐三本编程入门书籍。"),
        ("问题3", "请解释什么是函数式编程？"),
    ];

    println!("💬 准备并发发送 {} 个问题...", questions.len());
    let start_time = Instant::now();

    // 步骤4: 创建并发任务
    let tasks: Vec<_> = questions
        .into_iter()
        .enumerate()
        .map(|(index, (label, question))| {
            let client = client.clone();
            let question = question.to_string();
            let label = label.to_string();
            
            // 为每个任务创建独立的异步任务
            tokio::spawn(async move {
                println!("🔄 [{}] 发送请求: {}", label, question);
                
                match client.generate_with_stats(&question).await {
                    Ok(response) => {
                        println!("\n✅ [{}] 响应完成:", label);
                        println!("─────────────────────────────────");
                        println!("{}", response.content.trim());
                        println!("─────────────────────────────────");
                        println!("📊 统计信息: 用时 {}ms, 输入 {} tokens, 输出 {} tokens\n", 
                               response.stats.duration_ms,
                               response.stats.prompt_tokens.unwrap_or(0),
                               response.stats.completion_tokens.unwrap_or(0));
                        
                        Ok((index, label, response.content, response.stats))
                    }
                    Err(e) => {
                        eprintln!("❌ [{}] 请求失败: {}", label, e);
                        Err((index, label, e))
                    }
                }
            })
        })
        .collect();

    println!("⏳ 等待所有请求完成...\n");

    // 步骤5: 等待所有任务完成
    let results = join_all(tasks).await;
    
    let total_duration = start_time.elapsed();
    println!("🎉 所有请求完成! 总耗时: {:?}\n", total_duration);

    // 步骤6: 处理结果并统计
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_input_tokens = 0;
    let mut total_output_tokens = 0;
    let mut total_api_time = 0;

    for (task_index, task_result) in results.into_iter().enumerate() {
        match task_result {
            Ok(Ok((_index, label, content, stats))) => {
                successful_requests += 1;
                total_input_tokens += stats.prompt_tokens.unwrap_or(0);
                total_output_tokens += stats.completion_tokens.unwrap_or(0);
                total_api_time += stats.duration_ms;
                
                println!("📝 [{}] 最终结果摘要:", label);
                let summary = if content.chars().count() > 100 {
                    content.chars().take(100).collect::<String>() + "..."
                } else {
                    content
                };
                println!("   {}", summary.replace('\n', " "));
            }
            Ok(Err((_index, label, error))) => {
                failed_requests += 1;
                println!("❌ [{}] 失败: {}", label, error);
            }
            Err(join_error) => {
                failed_requests += 1;
                println!("❌ 任务 {} 执行失败: {}", task_index, join_error);
            }
        }
    }

    // 步骤7: 打印最终统计
    println!("\n📊 最终统计报告:");
    println!("─────────────────────────────────");
    println!("✅ 成功请求: {}", successful_requests);
    println!("❌ 失败请求: {}", failed_requests);
    println!("🔢 总输入 tokens: {}", total_input_tokens);
    println!("🔢 总输出 tokens: {}", total_output_tokens);
    println!("⏱️  API总耗时: {}ms", total_api_time);
    println!("⏱️  实际总耗时: {:?}", total_duration);
    
    if successful_requests > 0 {
        println!("📈 平均每请求耗时: {}ms", total_api_time / successful_requests as u64);
        println!("🚀 并发效率: {:.1}x (相比串行执行)", 
               total_api_time as f64 / total_duration.as_millis() as f64);
    }
    
    println!("\n🎯 并发聊天示例完成!");
    println!("\n📚 提示:");
    println!("   • 并发请求可以显著提高处理效率");
    println!("   • 每个请求使用独立的随机种子确保结果多样性");
    println!("   • 可以通过调整问题数量来测试不同的并发场景");
    println!("   • 注意API速率限制，避免过多并发请求");

    Ok(())
}