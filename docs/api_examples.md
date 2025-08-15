# NanoAI API 调用详细示例

本文档提供了 NanoAI 库所有 API 方法的详细使用示例和最佳实践。

## 📚 目录

- [基础配置](#基础配置)
- [核心 API 方法](#核心-api-方法)
- [高级用法](#高级用法)
- [错误处理](#错误处理)
- [性能优化](#性能优化)
- [最佳实践](#最佳实践)

## 基础配置

### 1. 默认配置

```rust
use nanoai::{Config, LLMClient};

// 使用默认配置（从环境变量加载）
let config = Config::from_env();
let client = LLMClient::new(config);
```

### 2. 自定义配置

```rust
use nanoai::Config;

// 构建器模式配置
let config = Config::builder()
    .api_key("your-api-key")
    .model("gpt-3.5-turbo")
    .base_url("https://api.openai.com/v1")
    .temperature(0.7)
    .max_tokens(1000)
    .timeout(30)
    .with_random_seed_auto()  // 自动生成随机种子
    .build();

let client = LLMClient::new(config);
```

### 3. 环境变量配置

```bash
# .env 文件
OPENAI_API_KEY=your-api-key
OPENAI_MODEL=gpt-3.5-turbo
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_TEMPERATURE=0.7
OPENAI_MAX_TOKENS=1000
OPENAI_TIMEOUT=30
```

## 核心 API 方法

### 1. `generate()` - 基础文本生成

**方法签名：**
```rust
pub async fn generate(&self, prompt: &str) -> Result<String, NanoError>
```

**使用示例：**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    // 简单的文本生成
    let response = client.generate("请解释什么是 Rust 编程语言？").await?;
    println!("AI 回复: {}", response);
    
    Ok(())
}
```

**适用场景：**
- 简单的问答
- 文本生成任务
- 不需要统计信息的场景

### 2. `generate_with_stats()` - 带统计信息的文本生成

**方法签名：**
```rust
pub async fn generate_with_stats(&self, prompt: &str) -> Result<ResponseWithStats, NanoError>
```

**使用示例：**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    let response = client.generate_with_stats("请写一个 Rust 函数来计算斐波那契数列").await?;
    
    println!("AI 回复: {}", response.content);
    println!("统计信息:");
    println!("  - 用时: {}ms", response.stats.duration_ms);
    println!("  - 输入 tokens: {}", response.stats.prompt_tokens.unwrap_or(0));
    println!("  - 输出 tokens: {}", response.stats.completion_tokens.unwrap_or(0));
    println!("  - 总 tokens: {}", response.stats.total_tokens.unwrap_or(0));
    
    Ok(())
}
```

**适用场景：**
- 需要监控 API 使用情况
- 性能分析和优化
- 成本控制和预算管理

### 3. `generate_with_context_stats()` - 多轮对话

**方法签名：**
```rust
pub async fn generate_with_context_stats(
    &self,
    system_prompt: &str,
    messages: &[Message],
) -> Result<ResponseWithStats, NanoError>
```

**使用示例：**
```rust
use nanoai::{Config, LLMClient, message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    // 构建对话历史
    let messages = vec![
        message("user", "我想学习 Rust 编程"),
        message("assistant", "很好的选择！Rust 是一门系统编程语言，以内存安全和高性能著称。"),
        message("user", "请推荐一些学习资源"),
    ];
    
    let response = client.generate_with_context_stats(
        "你是一个专业的编程导师，请提供准确、实用的建议。",
        &messages
    ).await?;
    
    println!("AI 回复: {}", response.content);
    println!("对话统计: {}ms, {} tokens", 
             response.stats.duration_ms,
             response.stats.total_tokens.unwrap_or(0));
    
    Ok(())
}
```

**适用场景：**
- 聊天机器人
- 多轮对话系统
- 上下文相关的问答

### 4. `generate_stream()` - 流式响应

**方法签名：**
```rust
pub async fn generate_stream(
    &self,
    prompt: &str,
) -> Result<impl Stream<Item = Result<String, NanoError>>, NanoError>
```

**使用示例：**
```rust
use futures::StreamExt;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    let mut stream = client.generate_stream("请写一个关于 Rust 的技术博客文章").await?;
    
    print!("AI 正在生成内容: ");
    io::stdout().flush().unwrap();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(content) => {
                print!("{}", content);
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("\n流式错误: {:?}", e);
                break;
            }
        }
    }
    
    println!("\n\n生成完成！");
    Ok(())
}
```

**适用场景：**
- 实时打字效果
- 长文本生成
- 用户体验优化

## 高级用法

### 1. 并发处理多个请求

```rust
use futures::future::join_all;
use tokio;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    let questions = vec![
        "请解释什么是机器学习？",
        "Rust 的所有权系统是如何工作的？",
        "什么是函数式编程的优势？",
        "请介绍一下区块链技术",
        "如何优化数据库查询性能？",
    ];
    
    let start_time = Instant::now();
    
    // 创建并发任务
    let tasks: Vec<_> = questions.into_iter().enumerate().map(|(i, question)| {
        let client = client.clone();
        tokio::spawn(async move {
            let task_start = Instant::now();
            let result = client.generate_with_stats(question).await;
            let task_duration = task_start.elapsed();
            (i, question, result, task_duration)
        })
    }).collect();
    
    // 等待所有任务完成
    let results = join_all(tasks).await;
    let total_duration = start_time.elapsed();
    
    // 统计信息
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_tokens = 0;
    
    // 处理结果
    for task_result in results {
        match task_result {
            Ok((index, question, Ok(response), task_duration)) => {
                successful_requests += 1;
                total_tokens += response.stats.total_tokens.unwrap_or(0);
                
                println!("\n=== 问题 {} ===", index + 1);
                println!("问题: {}", question);
                println!("回答: {}...", 
                         response.content.chars().take(100).collect::<String>());
                println!("统计: {}ms, {} tokens", 
                         response.stats.duration_ms,
                         response.stats.total_tokens.unwrap_or(0));
                println!("任务用时: {:?}", task_duration);
            }
            Ok((index, _question, Err(e), _)) => {
                failed_requests += 1;
                println!("问题 {} 失败: {}", index + 1, e);
            }
            Err(e) => {
                failed_requests += 1;
                println!("任务执行失败: {}", e);
            }
        }
    }
    
    // 最终统计报告
    println!("\n=== 并发处理统计报告 ===");
    println!("总用时: {:?}", total_duration);
    println!("成功请求: {}", successful_requests);
    println!("失败请求: {}", failed_requests);
    println!("总 tokens: {}", total_tokens);
    println!("平均每个请求用时: {:?}", 
             total_duration / (successful_requests + failed_requests) as u32);
    
    Ok(())
}
```

### 2. 智能重试机制

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn generate_with_retry(
    client: &LLMClient,
    prompt: &str,
    max_retries: u32,
) -> Result<ResponseWithStats, NanoError> {
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match client.generate_with_stats(prompt).await {
            Ok(response) => {
                if attempt > 0 {
                    println!("重试成功，第 {} 次尝试", attempt + 1);
                }
                return Ok(response);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    let delay = Duration::from_millis(1000 * (2_u64.pow(attempt)));
                    println!("请求失败，{}ms 后重试...", delay.as_millis());
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    let response = generate_with_retry(
        &client,
        "请解释量子计算的基本原理",
        3  // 最多重试 3 次
    ).await?;
    
    println!("最终回复: {}", response.content);
    Ok(())
}
```

## 错误处理

### 错误类型说明

```rust
use nanoai::NanoError;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let client = LLMClient::new(config);
    
    match client.generate("测试问题").await {
        Ok(response) => {
            println!("成功: {}", response);
        }
        Err(e) => {
            match e {
                NanoError::ConfigError(msg) => {
                    eprintln!("配置错误: {}", msg);
                    // 检查环境变量或配置文件
                }
                NanoError::NetworkError(msg) => {
                    eprintln!("网络错误: {}", msg);
                    // 检查网络连接或代理设置
                }
                NanoError::ApiError { code, message } => {
                    eprintln!("API 错误 {}: {}", code, message);
                    // 检查 API 密钥或请求参数
                }
                NanoError::ParseError(msg) => {
                    eprintln!("解析错误: {}", msg);
                    // API 响应格式异常
                }
                NanoError::TimeoutError => {
                    eprintln!("请求超时");
                    // 增加超时时间或检查网络
                }
                NanoError::RateLimitError => {
                    eprintln!("请求频率限制");
                    // 实现退避重试策略
                }
            }
        }
    }
}
```

## 性能优化

### 1. 连接池和客户端复用

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

// 全局客户端实例
static CLIENT: once_cell::sync::Lazy<Arc<LLMClient>> = once_cell::sync::Lazy::new(|| {
    let config = Config::from_env();
    Arc::new(LLMClient::new(config))
});

// 在多个地方复用客户端
async fn process_request(prompt: &str) -> Result<String, NanoError> {
    CLIENT.generate(prompt).await
}
```

### 2. 批量处理优化

```rust
use futures::stream::{self, StreamExt};

async fn batch_process(
    client: &LLMClient,
    prompts: Vec<String>,
    concurrency_limit: usize,
) -> Vec<Result<ResponseWithStats, NanoError>> {
    stream::iter(prompts)
        .map(|prompt| async move {
            client.generate_with_stats(&prompt).await
        })
        .buffer_unordered(concurrency_limit)
        .collect()
        .await
}
```

## 最佳实践

### 1. 配置管理

```rust
// 推荐：使用环境变量配置
let config = Config::from_env();

// 开发环境：使用 .env 文件
// 生产环境：使用系统环境变量或配置管理服务
```

### 2. 错误处理策略

```rust
// 推荐：详细的错误处理
match client.generate(prompt).await {
    Ok(response) => { /* 处理成功响应 */ }
    Err(NanoError::RateLimitError) => {
        // 实现退避重试
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 重试逻辑
    }
    Err(e) => {
        // 记录错误日志
        log::error!("AI 请求失败: {:?}", e);
        // 返回用户友好的错误信息
    }
}
```

### 3. 资源管理

```rust
// 推荐：合理设置超时和限制
let config = Config::builder()
    .timeout(30)  // 30秒超时
    .max_tokens(1000)  // 限制输出长度
    .temperature(0.7)  // 平衡创造性和准确性
    .build();
```

### 4. 监控和日志

```rust
use log::{info, warn, error};

let start = std::time::Instant::now();
let response = client.generate_with_stats(prompt).await?;
let duration = start.elapsed();

info!("AI 请求完成: {}ms, {} tokens", 
      duration.as_millis(),
      response.stats.total_tokens.unwrap_or(0));

if duration.as_secs() > 10 {
    warn!("AI 请求耗时过长: {:?}", duration);
}
```

---

## 总结

NanoAI 提供了简洁而强大的 API，支持从简单的文本生成到复杂的并发处理场景。通过合理使用这些 API 和最佳实践，可以构建高效、可靠的 AI 应用程序。

关键要点：
- 使用 `generate_with_stats()` 进行性能监控
- 利用 `generate_with_context_stats()` 实现多轮对话
- 通过 `generate_stream()` 提升用户体验
- 实现适当的错误处理和重试机制
- 合理配置并发限制和超时参数