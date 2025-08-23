//! API 数据结构模块

use serde::{Deserialize, Serialize};

// ================================================================================================
// API 请求结构
// ================================================================================================

/// 对话消息
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Message {
    /// 角色
    pub role: Role,
    /// 内容
    pub content: String,
}

/// 角色枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 系统
    System,
    /// 用户
    #[default]
    User,
    /// 机器人
    Assistant,
}

// ================================================================================================
// API 响应结构
// ================================================================================================

/// API 响应体
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CompletionResponse {
    /// 响应 ID
    #[serde(default)]
    pub id: String,
    /// 对话选择
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// 创建时间
    #[serde(default)]
    pub created: u64,
    /// 使用模型
    #[serde(default)]
    pub model: String,
    /// 系统指纹
    pub system_fingerprint: Option<String>,
    /// 对象类型
    #[serde(default)]
    pub object: String,
    /// token 使用情况
    #[serde(default)]
    pub usage: Usage,
}

/// 对话选择
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Choice {
    /// 结束原因
    #[serde(default)]
    pub finish_reason: String,
    /// 索引
    #[serde(default)]
    pub index: u32,
    /// 消息内容
    #[serde(default)]
    pub message: Message,
}

/// token 使用情况
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Usage {
    /// 完成 token 数量
    #[serde(default)]
    pub completion_tokens: u32,
    /// 提示 token 数量
    #[serde(default)]
    pub prompt_tokens: u32,
    /// 总 token 数量
    #[serde(default)]
    pub total_tokens: u32,
}

// ================================================================================================
// 流式 API 响应结构
// ================================================================================================

/// 流式 API 响应增量
#[derive(Debug, Deserialize, Serialize)]
pub struct Delta {
    /// 角色
    pub role: Option<Role>,
    /// 内容
    pub content: Option<String>,
}

/// 流式 API 响应体
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StreamCompletionResponse {
    /// 响应 ID
    pub id: String,
    /// 对话选择
    pub choices: Vec<StreamChoice>,
    /// 创建时间
    pub created: u64,
    /// 使用模型
    pub model: String,
    /// 系统指纹
    pub system_fingerprint: Option<String>,
    /// 对象类型
    pub object: String,
}

/// 流式对话选择
#[derive(Debug, Deserialize, Serialize)]
pub struct StreamChoice {
    /// 增量内容
    pub delta: Delta,
    /// 结束原因
    pub finish_reason: Option<String>,
    /// 索引
    pub index: u32,
}

// ================================================================================================
// 应用内部数据模型
// ================================================================================================

/// 请求统计信息
///
/// 记录 API 请求的详细统计数据，用于性能监控和分析
#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    /// 请求耗时（毫秒）
    pub duration_ms: u64,
    /// 输入 token 数量
    pub prompt_tokens: Option<u32>,
    /// 输出 token 数量
    pub completion_tokens: Option<u32>,
    /// 总 token 数量
    pub total_tokens: Option<u32>,
    /// 使用的模型名称
    pub model: String,
    /// 请求时间戳
    pub timestamp: Option<std::time::SystemTime>,
}

/// 带统计信息的响应结果
///
/// 包含生成的内容和详细的请求统计信息
#[derive(Debug)]
pub struct ResponseWithStats {
    /// 生成的文本内容
    pub content: String,
    /// 请求统计信息
    pub stats: RequestStats,
}