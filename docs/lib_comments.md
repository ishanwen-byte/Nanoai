# NanoAI 库注释文档

本文档包含从 `lib.rs` 文件中提取的所有注释内容。

## 库级别文档注释

### 主要特点
- **不可变性**: 所有配置对象都是不可变的，通过构建器模式创建新实例
- **纯函数**: 核心逻辑使用纯函数实现，便于测试和推理
- **错误处理**: 使用Result类型进行函数式错误处理
- **流式处理**: 支持实时流式文本生成
- **性能优化**: 预构建请求头，连接复用，智能重试机制

## 依赖导入注释

### 外部依赖导入
- `futures`: 异步流处理
- `log`: 日志记录
- `nanorand`: 轻量级随机数生成器
- `reqwest`: HTTP客户端
- `serde`: 序列化支持
- `serde_json`: JSON值类型

### 标准库导入
- `std::time`: 时间相关类型
- `thiserror`: 错误类型派生宏
- `tokio::time`: 异步睡眠

## 错误处理系统注释

采用函数式风格的错误类型设计，所有错误都是不可变的值对象。使用thiserror简化错误类型定义，自动实现Display和Error trait。

### NanoError 设计原则
- 每种错误都有明确的语义和上下文信息
- 支持错误链传播（使用\[from\]自动转换）
- 错误信息对用户友好，便于调试和监控

### 错误类型说明
- `Http`: HTTP请求失败（网络层错误）
- `Json`: JSON序列化/反序列化错误
- `Api`: API服务端返回的业务错误
- `Timeout`: 请求超时
- `NoContent`: 响应中没有内容
- `StreamError`: 流式处理错误
- `RateLimit`: API调用频率限制
- `Auth`: 身份验证失败
- `ModelNotFound`: 指定的模型不存在
- `InvalidRequest`: 请求参数无效
- `Config`: 配置错误

## 核心数据结构注释

采用函数式编程的不可变数据结构设计，所有字段都是值类型或不可变引用。

### Message 结构体
对话消息结构体，设计特点：
- 不可变：一旦创建就不能修改，确保数据一致性
- 可序列化：支持JSON序列化，便于网络传输和存储
- 可哈希：支持作为HashMap的键，便于缓存和去重
- 可比较：支持相等性比较，便于测试和调试

字段说明：
- `role`: 消息角色："system"（系统）、"user"（用户）、"assistant"（助手）
- `content`: 消息内容文本

### RequestStats 结构体
API请求性能统计信息，用于监控和优化API调用性能，支持以下指标：
- 请求耗时：用于性能分析和超时设置
- Token使用量：用于成本控制和配额管理
- 时间戳：用于日志记录和审计

字段说明：
- `duration_ms`: 请求总耗时（毫秒）
- `prompt_tokens`: 输入提示的Token数量
- `completion_tokens`: 生成完成的Token数量
- `total_tokens`: 总Token数量（prompt + completion）
- `model`: 使用的模型名称
- `timestamp`: 请求时间戳

### ResponseWithStats 结构体
包含统计信息的API响应，将生成的内容和性能统计信息封装在一起，便于上层应用进行性能监控和成本分析。

字段说明：
- `content`: 生成的文本内容
- `stats`: 请求统计信息

### Config 结构体
AI客户端配置结构体，采用函数式配置模式，支持以下特性：
- 不可变配置：配置一旦创建就不可修改，通过构建器模式创建新实例
- 环境变量集成：自动从.env文件和系统环境变量加载配置
- 合理默认值：提供生产环境可用的默认配置
- 链式构建：支持流畅的API风格进行配置定制

字段说明：
- `model`: AI模型名称（如：gpt-3.5-turbo, gpt-4, deepseek等）
- `system_message`: 系统消息，用于设定AI的行为和角色
- `temperature`: 生成温度（0.0-2.0），控制输出的随机性和创造性
- `top_p`: Top-p采样参数（0.0-1.0），控制词汇选择的多样性
- `max_tokens`: 最大生成Token数量，控制输出长度
- `timeout`: 请求超时时间，防止长时间等待
- `retries`: 最大重试次数，用于处理临时性错误
- `retry_delay`: 重试间隔时间，避免频繁重试造成服务压力
- `api_base`: API基础URL，支持不同的AI服务提供商
- `api_key`: API密钥，用于身份验证
- `random_seed`: 随机种子，用于确保输出的可重现性（可选）

### Config 默认值说明
默认配置适用于大多数使用场景：
- 使用免费的DeepSeek模型（性价比高，适合开发测试）
- 温度设为0.7（平衡创造性和一致性）
- 60秒超时（适合复杂推理任务）
- 3次重试机制（提高可靠性）
- 使用OpenRouter作为默认服务商（支持多种模型）

### Config::from_env() 方法
从环境变量加载配置

配置加载优先级：
1. 系统环境变量（最高优先级）
2. .env文件中的配置
3. 默认值（最低优先级）

支持的环境变量：
- `OPENAI_API_KEY` / `OPENROUTER_API_KEY` / `API_KEY`: API密钥
- `OPENROUTER_MODEL` / `MODEL`: 模型名称

返回配置好的Config实例，如果找不到API密钥则返回错误。

### 构建器模式方法
所有方法都是纯函数，返回新的Config实例而不修改原有实例。这确保了配置的不可变性，避免了并发访问时的数据竞争问题。

#### with_model()
设置AI模型名称
- 参数: `model` - 模型名称，如"gpt-4", "claude-3-sonnet", "deepseek-chat"等

#### with_api_key()
设置API密钥
- 参数: `api_key` - API密钥字符串
- 安全提示: 请确保API密钥的安全性，建议通过环境变量传递而不是硬编码

#### with_temperature()
设置生成温度
- 参数: `temperature` - 温度值（0.0-2.0）
  - 0.0: 最确定性的输出，适合事实性任务
  - 0.7: 平衡创造性和一致性，适合大多数场景
  - 1.0+: 更有创造性，适合创意写作

#### with_base_url()
设置API基础URL
- 参数: `api_base` - API基础URL，支持不同的服务提供商
  - OpenAI: https://api.openai.com/v1
  - OpenRouter: https://api.openrouter.com/v1
  - 自定义代理或本地服务

#### with_random_seed()
设置固定的随机种子
- 参数: `seed` - 随机种子值
- 用途: 设置固定种子可以确保相同输入产生相同输出，适用于：
  - 单元测试和回归测试
  - 需要可重现结果的场景
  - 调试和问题排查

#### with_random_seed_auto()
自动生成随机种子
使用高质量的随机数生成器（WyRand）自动生成种子值，确保每次调用都产生不同的随机行为。

## API响应类型定义注释

用于反序列化不同AI服务商的API响应，支持OpenAI兼容格式。

### CompletionResponse
完整的API响应结构体，对应OpenAI Chat Completions API的响应格式，包含生成的选择项、使用统计和模型信息。

字段说明：
- `choices`: 生成的选择项列表，通常只包含一个选择
- `usage`: Token使用统计信息，用于成本计算
- `model`: 实际使用的模型名称

### CompletionChoice
单个生成选择项（非流式模式），包含完整的响应消息和完成状态信息。

字段说明：
- `message`: 完整的响应消息
- `finish_reason`: 完成原因："stop"（正常完成）、"length"（达到长度限制）等

### CompletionMessage
API响应中的消息结构体

字段说明：
- `content`: 生成的文本内容
- `role`: 消息角色，通常为"assistant"

### Usage
Token使用统计信息，用于监控API使用量和成本控制。

字段说明：
- `prompt_tokens`: 输入提示使用的Token数量
- `completion_tokens`: 生成完成使用的Token数量
- `total_tokens`: 总Token数量（prompt + completion）

### StreamResponse
流式响应结构体，用于处理Server-Sent Events (SSE) 格式的流式响应。

字段说明：
- `choices`: 流式选择项列表
- `model`: 使用的模型名称
- `usage`: Token使用统计（通常在最后一个流式块中提供）

### StreamChoice
流式响应中的单个选择项

字段说明：
- `delta`: 增量消息内容
- `finish_reason`: 完成原因（仅在最后一个块中提供）
- `index`: 选择项索引

### StreamDelta
流式响应中的增量内容

字段说明：
- `content`: 本次流式块的文本内容
- `role`: 角色信息（通常只在第一个块中提供）

## 函数式LLM客户端注释

采用函数式编程范式设计的AI客户端，具有以下特点：
- 不可变性：客户端配置一旦创建就不可修改
- 纯函数：所有方法都是纯函数或具有明确的副作用边界
- 组合性：支持方法链式调用和函数组合
- 错误处理：使用Result类型进行函数式错误处理

### LLMClient 结构体
轻量级LLM客户端

核心设计原则：
- **性能优化**: 预构建请求头，连接复用，智能重试
- **类型安全**: 强类型配置，编译时错误检查
- **可观测性**: 内置请求统计和日志记录
- **兼容性**: 支持OpenAI兼容的多种AI服务商

字段说明：
- `client`: HTTP客户端实例，配置了超时和TLS设置
- `config`: 不可变的客户端配置
- `headers`: 预构建的HTTP请求头，避免每次请求时重复构建

### LLMClient::new() 方法
创建新的LLM客户端实例

初始化过程包括：
1. 创建配置了超时和TLS的HTTP客户端
2. 预构建认证请求头，提高请求性能
3. 记录模型初始化日志，便于调试

参数：
- `config` - 客户端配置，包含API密钥、模型名称、超时设置等

性能优化：
- 使用连接复用减少握手开销
- 预构建请求头避免重复序列化
- 配置合理的连接和请求超时

### generate() 方法
生成文本响应（简单接口）

这是最简单的文本生成接口，适用于单轮对话场景。内部会自动添加系统消息，并处理完整的请求-响应流程。

参数：
- `prompt` - 用户输入的提示文本

返回：
返回AI生成的文本内容，不包含统计信息

错误处理：
- 网络错误：连接超时、DNS解析失败等
- API错误：认证失败、模型不存在、请求格式错误等
- 解析错误：响应格式不正确、JSON解析失败等

### generate_with_stats() 方法
生成文本响应并返回详细统计信息

相比简单的generate方法，此方法额外返回性能统计信息，包括Token使用量、请求耗时等，适用于需要监控API使用情况的场景。

参数：
- `prompt` - 用户输入的提示文本

返回：
返回包含生成内容和统计信息的ResponseWithStats结构体：
- content: 生成的文本内容
- stats: 包含Token使用量、耗时、模型信息等统计数据

用途：
- 成本监控：跟踪Token使用量，控制API调用成本
- 性能分析：监控请求耗时，优化应用性能
- 使用统计：记录API调用情况，用于分析和报告

### generate_with_context_stats() 方法
带上下文和统计信息的生成方法（精简版）

参数：
- `system_msg` - 系统消息
- `messages` - 消息列表

返回：
返回包含生成内容和统计信息的ResponseWithStats结构体