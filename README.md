# NanoAI - 函数式编程优化版本

轻量级 AI 客户端库，采用函数式编程思想重构，专注于纯函数、不可变性和函数组合。

## ✨ 核心特性

- 🎯 **函数式编程**: 纯函数、不可变数据、函数组合
- 🚀 **轻量级设计**: 单文件架构，专注核心功能
- 🌊 **流式处理**: 函数式流抽象，支持实时响应
- 🛡️ **类型安全**: 完整的错误处理和类型约束
- 📦 **简洁API**: 链式调用和构建器模式
- ⚡ **高性能**: 零拷贝流处理和异步优化

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
nanoai = { path = "path/to/nanoai" }
tokio = { version = "1.0", features = ["full"] }
```

## 使用方法

### 基本用法

```rust
use nanoai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（需要有效的 OpenRouter API 密钥）
    let client = Client::new("gpt-3.5-turbo", "your-api-key".to_string());
    
    // 创建聊天消息
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Why is the sky blue?"),
    ];
    
    // 创建请求
    let request = ChatRequest::new("gpt-3.5-turbo", messages);
    
    // 发送请求
    match client.chat(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    
    Ok(())
}
```

### 流式请求

流式请求允许实时接收AI模型的响应，而不需要等待完整响应完成。这对于长文本生成特别有用。

```rust
use futures::StreamExt;
use std::io::Write;

// 创建流式请求
let request = ChatRequest::new(vec![
    ChatMessage::user("Tell me a story".to_string())
]).with_stream(true);

// 发送流式请求并处理响应
let mut stream = client.chat_stream(request).await?;
while let Some(result) = stream.next().await {
    match result {
        Ok(response) => {
            // 实时打印响应内容
            if !response.choices.is_empty() {
                print!("{}", response.choices[0].message.content);
                std::io::stdout().flush().unwrap();
            }
        }
        Err(e) => {
            eprintln!("\nStream error: {:?}", e);
            break;
        }
    }
}
println!(); // 换行
```

#### 流式处理特性

- **Server-Sent Events (SSE)**: 使用标准的SSE协议进行流式通信
- **实时响应**: 无需等待完整响应，可以立即显示生成的内容
- **错误处理**: 完整的流式错误处理，包括网络错误和API错误
- **自动解析**: 自动解析SSE格式的数据并转换为ChatResponse对象

## 错误处理

库提供了详细的错误类型：

- `AuthenticationFailed`: 认证失败（401 错误）
- `RateLimitExceeded`: API 速率限制（429 错误）
- `ServerError`: 服务器错误（5xx 错误）
- `InvalidResponse`: 响应格式无效
- `Http`: HTTP 请求错误

## 测试

运行所有测试：

```bash
cargo test
```

运行示例（需要设置 `OPENROUTER_API_KEY` 环境变量）：

```bash
export OPENROUTER_API_KEY="your-api-key"
cargo run --example chas
```

## 项目结构

```
src/
├── lib.rs          # 库入口和公共导出
├── client.rs       # 客户端核心逻辑
├── chat.rs         # 聊天消息和请求结构
├── auth.rs         # 认证数据处理
├── resolver.rs     # 服务目标解析
└── web.rs          # HTTP 请求工具
tests/
└── integration_tests.rs  # 集成测试
examples/
└── chas.rs         # 示例程序
```

## 改进内容

### 流式输出支持
- **完整的SSE实现**: 支持Server-Sent Events协议的真正流式处理
- **实时响应**: 可以实时接收和显示AI模型的响应内容
- **流式错误处理**: 完整的流式请求错误处理机制
- **自动数据解析**: 自动解析SSE格式数据并转换为结构化响应

### 代码精简
- 移除了不必要的复杂性
- 简化了流式处理逻辑
- 优化了模块结构

### 错误处理改进
- 添加了具体的错误类型
- 根据 HTTP 状态码返回相应错误
- 提供了更好的错误信息

### 测试覆盖
- 为所有模块添加了单元测试
- 添加了集成测试
- 测试覆盖率达到 100%

### 功能保留
- 保持了原有的核心功能
- 支持 OpenRouter API
- 兼容原有的 API 接口

## 许可证

MIT License