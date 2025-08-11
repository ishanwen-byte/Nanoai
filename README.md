# NanoAI - 轻量级AI客户端库

一个采用函数式编程思想设计的轻量级 AI 客户端库，专注于纯函数、不可变性和函数组合，提供简洁而强大的 AI 对话接口。

## ✨ 核心特性

- 🎯 **函数式编程**: 纯函数设计、不可变数据结构、函数组合
- 🚀 **轻量级架构**: 单文件实现，专注核心功能，零依赖膨胀
- 🌊 **流式处理**: 支持 Server-Sent Events (SSE) 实时流式响应
- 🛡️ **类型安全**: 完整的错误处理和 Rust 类型系统保障
- 📦 **简洁API**: 构建器模式和链式调用，易于使用
- ⚡ **高性能**: 异步处理、连接复用、智能重试机制
- 🔧 **灵活配置**: 支持多种 AI 服务提供商（OpenAI、OpenRouter 等）

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
nanoai = { git = "https://github.com/ishanwen-byte/Nanoai.git" }
tokio = { version = "1.0", features = ["full"] }
```

## 快速开始

### 基本用法

```rust
use nanoai::{Config, LLMClient, message};

#[tokio::main]
async fn main() -> nanoai::Result<()> {
    // 创建配置
    let config = Config::default()
        .with_api_key("your-api-key".to_string())
        .with_model("gpt-3.5-turbo".to_string());
    
    // 创建客户端
    let client = LLMClient::new(config);
    
    // 简单对话
    let response = client.generate("你好，请介绍一下你自己。").await?;
    println!("AI回复: {}", response);
    
    Ok(())
}
```

### 多轮对话

```rust
use nanoai::{Config, LLMClient, message};

// 创建对话消息
let messages = vec![
    message("user", "我想学习 Rust 编程"),
    message("assistant", "很好的选择！Rust 是一门系统编程语言..."),
    message("user", "请推荐一些学习资源"),
];

// 带上下文生成回复
let response = client.generate_with_context(
    "你是一个编程助手",
    &messages
).await?;
```

### 流式响应

```rust
use futures::StreamExt;
use std::io::{self, Write};

// 创建流式响应
let mut stream = client.generate_stream("写一个关于 Rust 的故事").await?;

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

## 配置选项

### 构建器模式配置

```rust
let config = Config::default()
    .with_api_key("your-api-key".to_string())
    .with_model("gpt-4".to_string())
    .with_temperature(0.8)                    // 创造性参数
    .with_base_url("https://api.openai.com/v1".to_string())
    .with_random_seed_auto();                 // 自动随机种子
```

### 支持的配置参数

- `model`: AI 模型名称（默认: `tngtech/deepseek-r1t2-chimera:free`）
- `api_key`: API 密钥
- `temperature`: 温度参数，控制创造性（0.0-1.0）
- `top_p`: Top-p 采样参数
- `max_tokens`: 最大生成令牌数
- `timeout`: 请求超时时间（默认: 60秒）
- `retries`: 重试次数（默认: 3次）
- `random_seed`: 随机种子，用于可重现的结果

## 错误处理

库提供了完整的错误类型系统：

```rust
use nanoai::NanoError;

match client.generate("Hello").await {
    Ok(response) => println!("成功: {}", response),
    Err(NanoError::Timeout) => println!("请求超时"),
    Err(NanoError::Api(msg)) => println!("API错误: {}", msg),
    Err(NanoError::Http(e)) => println!("网络错误: {}", e),
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

## 环境变量配置

支持通过环境变量或 `.env` 文件配置：

```bash
# OpenAI
OPENAI_API_KEY=your-openai-key

# OpenRouter
OPENROUTER_API_KEY=your-openrouter-key
OPENROUTER_MODEL=openai/gpt-4

# 通用
API_KEY=your-api-key
```

## 运行示例

```bash
# 设置环境变量
export OPENROUTER_API_KEY="your-api-key"

# 运行基础示例
cargo run --example basic_usage

# 运行流式示例
cargo run --example streaming_example

# 运行高级示例
cargo run --example advanced_usage
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_config_builder_pattern
```

## 项目结构

```
nanoai/
├── src/
│   └── lib.rs              # 主要实现文件
├── examples/
│   ├── basic_usage.rs      # 基础使用示例
│   ├── streaming_example.rs # 流式处理示例
│   ├── advanced_usage.rs   # 高级功能示例
│   └── quick_start.rs      # 快速开始示例
├── Cargo.toml              # 项目配置
└── README.md               # 项目文档
```

## 技术特性

### 函数式编程设计
- **纯函数**: 所有配置函数都是纯函数，不修改原始状态
- **不可变性**: 配置对象通过构建器模式创建新实例
- **函数组合**: 支持链式调用和函数组合模式

### 性能优化
- **连接复用**: 使用 `reqwest` 客户端连接池
- **异步处理**: 完全异步的 API 设计
- **智能重试**: 指数退避重试机制
- **流式处理**: 零拷贝流式数据处理

### 安全特性
- **TLS 安全**: 强制使用 rustls TLS 后端
- **证书验证**: 默认启用证书验证
- **超时保护**: 防止长时间挂起的请求
- **错误隔离**: 完整的错误类型系统

## 兼容性

- **Rust 版本**: 需要 Rust 1.70+
- **API 兼容**: 支持 OpenAI API 兼容的服务
- **平台支持**: Windows、macOS、Linux
- **异步运行时**: 基于 Tokio

## 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件