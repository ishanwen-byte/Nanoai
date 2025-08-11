# NanoAI 示例程序

本目录包含了 NanoAI 库的各种使用示例，展示了如何在不同场景下使用这个轻量级 AI 客户端库。

## 📋 示例列表

### 1. 基础使用示例 (`basic_usage.rs`)
展示 NanoAI 的基本功能：
- ✅ 基础配置和简单对话
- ⚙️ 自定义配置（温度、模型等）
- 💬 多轮对话
- 🌊 流式响应
- 🛡️ 错误处理

### 2. 高级使用示例 (`advanced_usage.rs`)
展示更复杂的使用场景：
- ⚡ 并发处理多个请求
- 📦 批量文本处理
- 🧠 智能对话系统
- 📊 性能测试和监控
- 🔬 不同模型比较

### 3. 流式处理示例 (`streaming_example.rs`)
专门展示流式响应的各种用法：
- 🌊 基础流式响应
- ⌨️ 实时打字效果
- 💬 流式对话
- 🔄 流式内容处理和统计
- 🛡️ 流式错误处理

## 🚀 运行示例

### 前置条件

1. **设置 API 密钥**
   ```bash
   # Windows PowerShell
   $env:OPENAI_API_KEY="your-api-key-here"
   
   # Windows CMD
   set OPENAI_API_KEY=your-api-key-here
   
   # 或者创建 .env 文件
   echo OPENAI_API_KEY=your-api-key-here > .env
   ```

2. **安装依赖**
   ```bash
   cargo build
   ```

### 运行示例

```bash
# 基础使用示例
cargo run --example basic_usage

# 高级使用示例
cargo run --example advanced_usage

# 流式处理示例
cargo run --example streaming_example
```

### 启用日志输出

```bash
# 启用详细日志
RUST_LOG=debug cargo run --example basic_usage

# 只显示错误和警告
RUST_LOG=warn cargo run --example advanced_usage
```

## 📖 代码说明

### 基本用法

```rust
use nanoai::{Config, LLMClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建配置
    let config = Config::default()
        .with_api_key("your-api-key".to_string())
        .with_model("gpt-3.5-turbo".to_string());
    
    // 创建客户端
    let client = LLMClient::new(config);
    
    // 生成回答
    let response = client.generate("Hello, AI!").await?;
    println!("AI: {}", response);
    
    Ok(())
}
```

### 流式响应

```rust
use futures::StreamExt;

// 创建流式响应
let mut stream = client.generate_stream("Tell me a story").await?;

// 逐块处理响应
while let Some(chunk_result) = stream.next().await {
    match chunk_result {
        Ok(chunk) => print!("{}", chunk),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 多轮对话

```rust
use nanoai::message;

// 构建对话历史
let messages = vec![
    message("user", "What is Rust?"),
    message("assistant", "Rust is a systems programming language..."),
    message("user", "Can you give me an example?"),
];

// 带上下文生成回答
let response = client
    .generate_with_context("You are a helpful programming tutor.", &messages)
    .await?;
```

## ⚙️ 配置选项

```rust
let config = Config::default()
    .with_api_key("your-key".to_string())
    .with_model("gpt-4".to_string())           // 模型选择
    .with_temperature(0.7);                     // 创造性 (0.0-2.0)
```

支持的配置项：
- `model`: AI 模型名称
- `api_key`: API 密钥
- `temperature`: 响应的随机性/创造性
- `top_p`: 核采样参数
- `max_tokens`: 最大响应长度
- `timeout`: 请求超时时间
- `retries`: 重试次数
- `retry_delay`: 重试延迟
- `api_base`: API 基础 URL
- `system_message`: 默认系统消息

## 🔧 支持的模型

- OpenAI GPT 系列：`gpt-3.5-turbo`, `gpt-4`, `gpt-4o-mini` 等
- 其他 OpenAI 兼容的 API 服务

## 🛡️ 错误处理

```rust
match client.generate("Hello").await {
    Ok(response) => println!("Success: {}", response),
    Err(e) => match e {
        nanoai::NanoError::Api(msg) => println!("API Error: {}", msg),
        nanoai::NanoError::Http(_) => println!("Network Error"),
        nanoai::NanoError::Timeout => println!("Request Timeout"),
        _ => println!("Other Error: {}", e),
    }
}
```

## 📊 性能提示

1. **并发请求**: 使用 `tokio::spawn` 并发处理多个请求
2. **流式响应**: 对于长文本生成，使用流式 API 获得更好的用户体验
3. **错误重试**: 库内置了自动重试机制
4. **连接复用**: 客户端会自动复用 HTTP 连接

## 🐛 故障排除

### 常见问题

1. **API 密钥错误**
   ```
   Error: API error: HTTP 401: Unauthorized
   ```
   解决：检查 API 密钥是否正确设置

2. **网络超时**
   ```
   Error: Request timed out
   ```
   解决：增加超时时间或检查网络连接

3. **模型不存在**
   ```
   Error: API error: HTTP 404: Model not found
   ```
   解决：检查模型名称是否正确

### 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug cargo run --example basic_usage

# 查看网络请求
RUST_LOG=reqwest=debug cargo run --example basic_usage
```

## 📚 更多资源

- [NanoAI 文档](../README.md)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Tokio 文档](https://tokio.rs/)
- [OpenAI API 文档](https://platform.openai.com/docs/api-reference)

## 🤝 贡献

欢迎提交新的示例或改进现有示例！请确保：

1. 代码风格一致
2. 包含适当的错误处理
3. 添加必要的注释
4. 测试示例可以正常运行

---

💡 **提示**: 这些示例展示了 NanoAI 的各种功能，你可以根据自己的需求进行修改和扩展。