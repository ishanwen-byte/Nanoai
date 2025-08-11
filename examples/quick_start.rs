//! NanoAI 快速入门示例
//! 最简单的使用方式，帮助用户快速上手

use nanoai::{Config, LLMClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 NanoAI 快速入门\n");

    // 步骤1: 从.env文件获取API密钥和配置
    dotenv::dotenv().ok();

    let (api_key, base_url, model) = if let (Ok(key), Ok(model)) = (
        dotenv::var("OPENROUTER_API_KEY"),
        dotenv::var("OPENROUTER_MODEL"),
    ) {
        println!("🌐 使用 OpenRouter 配置");
        (key, Some("https://openrouter.ai/api/v1".to_string()), model)
    } else if let Ok(key) = dotenv::var("OPENAI_API_KEY").or_else(|_| dotenv::var("API_KEY")) {
        println!("🤖 使用 OpenAI 配置");
        (key, None, "gpt-3.5-turbo".to_string())
    } else {
        println!("❌ 未找到API密钥！");
        println!("请在 .env 文件中设置以下环境变量之一:");
        println!("   OPENROUTER_API_KEY=your_openrouter_key");
        println!("   OPENROUTER_MODEL=your_model_name");
        println!("或者:");
        println!("   OPENAI_API_KEY=your_openai_key");
        return Ok(());
    };

    println!("✅ API密钥已设置");
    println!("🔧 使用模型: {model}");

    // 步骤2: 创建配置
    println!("🔧 创建配置...");
    let mut config = Config::default().with_api_key(api_key).with_model(model);

    if let Some(url) = base_url {
        config = config.with_base_url(url);
        println!("🌐 使用自定义API端点: OpenRouter");
    }

    // 步骤3: 创建客户端
    println!("🤖 创建AI客户端...");
    let client = LLMClient::new(config);

    // 步骤4: 发送第一个请求
    println!("💬 发送第一个请求...");

    match client.generate("你好！请简单介绍一下你自己。").await {
        Ok(response) => {
            println!("\n🤖 AI回复:");
            println!("─────────────────────────────────");
            println!("{response}");
            println!("─────────────────────────────────");
        }
        Err(e) => {
            println!("❌ 请求失败: {e}");
            println!("\n💡 可能的解决方案:");
            println!("   1. 检查API密钥是否正确");
            println!("   2. 检查网络连接");
            println!("   3. 确认API配额是否充足");
            return Ok(());
        }
    }

    // 步骤5: 尝试另一个问题
    println!("\n🔄 尝试另一个问题...");

    match client.generate("请用一句话解释什么是人工智能。").await {
        Ok(response) => {
            println!("\n🤖 AI回复: {response}");
        }
        Err(e) => {
            println!("❌ 第二个请求失败: {e}");
        }
    }

    // 成功完成
    println!("\n🎉 恭喜！你已经成功使用了 NanoAI");
    println!("\n📚 下一步可以尝试:");
    println!("   • 运行更多示例: cargo run --example basic_usage");
    println!("   • 尝试流式响应: cargo run --example streaming_example");
    println!("   • 查看高级功能: cargo run --example advanced_usage");
    println!("   • 阅读文档: examples/README.md");

    Ok(())
}
