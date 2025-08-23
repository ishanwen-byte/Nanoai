# NanoAI - 轻量级 LLM 客户端库

一个专为大语言模型 API 设计的轻量级 Rust 客户端库，提供简洁、函数式的接口来与各种大语言模型进行交互。

## ✨ 核心特性

- 🚀 **异步支持**: 基于 `tokio` 的完全异步实现，性能卓越
- 🔄 **流式响应**: 支持实时流式文本生成，提供即时反馈
- 📊 **统计信息**: 可选的详细请求统计和性能监控
- 🔧 **灵活配置**: 支持环境变量和 Builder 模式，轻松定制客户端
- 🛡️ **错误处理**: 完善的错误类型和基于 `backoff` 的自动重试机制
- 🎯 **函数式设计**: 遵循 Rust 函数式编程最佳实践，代码简洁、可预测
- ⚡ **高性能**: 连接池复用、并发控制、智能重试机制
- 🌐 **多平台支持**: Windows、macOS、Linux 全平台兼容

## 📦 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
nanoai = { git = "https://github.com/ishanwen-byte/Nanoai.git" }
tokio = { version = "1.0", features = ["full"] }
```

## 🚀 快速开始

### 基本用法

```rust
use nanoai::client::LLMClient;
use nanoai::config::Config;
use nanoai::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量加载配置 (需要设置 OPENROUTER_API_KEY)
    let config = Config::from_env()?;
    let client = LLMClient::new(config);
    
    // 简单对话
    let response = client.generate("你好，请介绍一下你自己。").await?;
    println!("AI回复: {}", response);
    
    Ok(())
}
```

### 手动配置

```rust
use nanoai::{client::LLMClient, config::Config};

// 使用 Builder 模式创建配置
let config = Config::default()
    .with_api_key("your-api-key".to_string())
    .with_model("openai/gpt-3.5-turbo".to_string())
    .with_temperature(0.8)
    .with_max_tokens(1000);

let client = LLMClient::new(config);
```

### 多轮对话

```rust
use nanoai::{client::LLMClient, types::Message, utils::message};

// 创建对话消息
let messages = vec![
    message("user", "我想学习 Rust 编程"),
    message("assistant", "很好的选择！Rust 是一门系统编程语言..."),
    message("user", "请推荐一些学习资源"),
];

// 批量生成回复
let response = client.batch_generate(&messages).await?;
println!("AI回复: {}", response);
```

### 带统计信息的调用

```rust
// 获取详细的请求统计信息
let response = client.generate_with_stats("解释什么是函数式编程").await?;

println!("AI回复: {}", response.content);
println!("统计信息:");
println!("  用时: {}ms", response.stats.duration_ms);
if let Some(tokens) = response.stats.prompt_tokens {
    println!("  输入 tokens: {}", tokens);
}
if let Some(tokens) = response.stats.completion_tokens {
    println!("  输出 tokens: {}", tokens);
}
```

### 流式响应

```rust
use futures::StreamExt;
use std::io::{self, Write};

// 创建流式响应
let mut stream = client.stream_generate("写一个关于 Rust 的故事").await?;

// 实时处理响应
while let Some(result) = stream.next().await {
    match result {
        Ok(content) => {
            print!("{}", content);
            io::stdout().flush().unwrap();
        }
        Err(e) => {
            eprintln!("流式错误: {:?}", e);
            break;
        }
    }
}
println!(); // 换行
```

### 并发处理

```rust
use futures::future::join_all;
use tokio;

// 准备多个问题
let questions = vec![
    "请用一句话解释什么是人工智能？",
    "请推荐三本编程入门书籍。",
    "请解释什么是函数式编程？",
];

// 并发处理所有问题
let tasks: Vec<_> = questions.into_iter().enumerate().map(|(i, question)| {
    let client = client.clone();
    tokio::spawn(async move {
        let result = client.generate_with_stats(question).await;
        (i, question, result)
    })
}).collect();

// 等待所有任务完成
let results = join_all(tasks).await;

// 处理结果
for task_result in results {
    match task_result {
        Ok((index, question, Ok(response))) => {
            println!("问题 {}: {}", index + 1, question);
            println!("回答: {}", response.content);
            println!("统计: {}ms\n", response.stats.duration_ms);
        }
        Ok((index, _question, Err(e))) => {
            println!("问题 {} 失败: {}", index + 1, e);
        }
        Err(e) => {
            println!("任务执行失败: {}", e);
        }
    }
}
```

## ⚙️ 配置选项

### 环境变量配置

支持通过环境变量或 `.env` 文件配置：

```bash
# OpenRouter (推荐)
OPENROUTER_API_KEY=your-openrouter-key
OPENROUTER_MODEL=openai/gpt-4

# OpenAI
OPENAI_API_KEY=your-openai-key

# 通用配置
API_KEY=your-api-key
MODEL=your-model-name
TEMPERATURE=0.7
MAX_TOKENS=1000
```

### Builder 模式配置

```rust
let config = Config::default()
    .with_api_key("your-api-key".to_string())
    .with_model("openai/gpt-4".to_string())
    .with_temperature(0.8)                    // 创造性参数 (0.0-2.0)
    .with_top_p(0.9)                         // Top-p 采样
    .with_max_tokens(2000)                   // 最大生成令牌数
    .with_timeout(std::time::Duration::from_secs(120))  // 请求超时
    .with_api_base("https://openrouter.ai/api/v1".to_string());
```

### 支持的配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `api_key` | String | - | API 密钥 |
| `model` | String | `"tngtech/deepseek-r1t2-chimera:free"` | 模型名称 |
| `temperature` | f32 | 0.7 | 温度参数，控制创造性 (0.0-2.0) |
| `top_p` | f32 | 1.0 | Top-p 采样参数 |
| `max_tokens` | u32 | 1000 | 最大生成令牌数 |
| `timeout` | Duration | 60秒 | 请求超时时间 |
| `api_base` | String | `"https://openrouter.ai/api/v1"` | API 基础 URL |
| `random_seed` | u64 | 随机 | 随机种子，用于可重现的结果 |

## 🛡️ 错误处理

库提供了完整的错误类型系统：

```rust
use nanoai::error::NanoError;

match client.generate("Hello").await {
    Ok(response) => println!("成功: {}", response),
    Err(NanoError::Timeout) => println!("请求超时"),
    Err(NanoError::Api(msg)) => println!("API错误: {}", msg),
    Err(NanoError::Http(e)) => println!("网络错误: {}", e),
    Err(NanoError::Json(e)) => println!("JSON解析错误: {}", e),
    Err(e) => println!("其他错误: {:?}", e),
}
```

### 错误类型

- `Http`: HTTP 请求错误
- `Json`: JSON 解析错误
- `Api`: API 服务错误
- `Timeout`: 请求超时
- `NoContent`: 响应无内容
- `StreamError`: 流式处理错误
- `InvalidRequest`: 无效请求参数

## 📖 示例程序

项目提供了丰富的示例程序，涵盖各种使用场景：

```bash
# 基础同步调用
cargo run --example basic_sync

# 异步调用
cargo run --example async_calls

# 流式响应
cargo run --example streaming

# 并发处理
cargo run --example concurrent

# 错误处理
cargo run --example error_handling

# 自定义配置
cargo run --example custom_config

# 性能基准测试
cargo run --example performance_benchmark
```

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行示例测试
cargo test --examples

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_config_builder_pattern

## 📖 示例程序

项目提供了以下示例程序：

```bash
# 基本使用
cargo run --example basic_usage

# 流式响应
cargo run --example streaming

# 批量生成
cargo run --example batch
```

这些示例演示了库的基本功能、流式处理和批量操作。
```

## 📁 项目结构

```
nanoai/
├── src/
│   ├── lib.rs              # 库入口和公共函数
│   ├── client.rs           # LLM 客户端核心实现
│   ├── config.rs           # 配置管理
│   ├── error.rs            # 错误类型定义
│   ├── stream.rs           # 流式处理
│   ├── types.rs            # 数据类型定义
│   └── utils.rs            # 工具函数
├── examples/
│   ├── basic_sync.rs       # 基础同步调用示例
│   ├── async_calls.rs      # 异步调用示例
│   ├── streaming.rs        # 流式处理示例
│   ├── concurrent.rs       # 并发处理示例
│   ├── error_handling.rs   # 错误处理示例
│   ├── custom_config.rs    # 配置自定义示例
│   └── performance_benchmark.rs # 性能基准测试
├── docs/                   # 文档目录
├── Cargo.toml              # 项目配置
├── pre_quality.ps1         # 质量控制脚本
└── README.md               # 项目文档
```

## 🔧 技术特性

### 函数式编程设计
- **纯函数**: 所有配置函数都是纯函数，不修改原始状态
- **不可变性**: 配置对象通过构建器模式创建新实例
- **函数组合**: 支持链式调用和函数组合模式
- **错误传播**: 使用 `Result` 类型进行优雅的错误处理

### 性能优化
- **连接复用**: 使用 `reqwest` 客户端连接池
- **异步处理**: 完全异步的 API 设计
- **并发控制**: 基于信号量的并发请求限制
- **流式处理**: 零拷贝流式数据处理
- **智能重试**: 指数退避重试机制

### 安全特性
- **TLS 安全**: 强制使用安全的 HTTPS 连接
- **超时保护**: 防止长时间挂起的请求
- **错误隔离**: 完整的错误类型系统
- **内存安全**: Rust 的所有权系统保证内存安全

## 🌍 兼容性

- **Rust 版本**: 需要 Rust 1.70+
- **API 兼容**: 支持 OpenAI API 兼容的服务
- **平台支持**: Windows、macOS、Linux
- **异步运行时**: 基于 Tokio
- **支持的服务**: OpenRouter、OpenAI、以及其他兼容 OpenAI API 的服务

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 开发指南

```bash
# 克隆仓库
git clone https://github.com/ishanwen-byte/Nanoai.git
cd Nanoai

# 运行质量控制检查
.\pre_quality.ps1

# 运行测试
cargo test

# 运行示例
cargo run --example basic_sync
```

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

感谢所有贡献者和 Rust 社区的支持！

---

**NanoAI** - 让 AI 集成变得简单而优雅 🚀