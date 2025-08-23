//! 错误处理模块

use thiserror::Error;

/// NanoAI 库的统一错误类型
///
/// 提供了完整的错误分类，便于上层应用进行精确的错误处理
#[derive(Debug, Error)]
pub enum NanoError {
    /// HTTP 请求相关错误
    #[error("HTTP请求失败: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON处理错误: {0}")]
    Json(String),

    /// API 服务端错误
    #[error("API错误: {0}")]
    Api(String),

    /// 请求超时错误
    #[error("请求超时")]
    Timeout,

    /// 响应内容为空
    #[error("响应内容为空")]
    NoContent,

    /// 流处理相关错误
    #[error("流处理错误: {0}")]
    StreamError(String),

    /// API 请求频率限制
    #[error("请求频率超限: {0}")]
    RateLimit(String),

    /// 身份验证失败
    #[error("身份验证失败: {0}")]
    Auth(String),

    /// 指定的模型不存在
    #[error("模型不存在: {0}")]
    ModelNotFound(String),

    /// 请求参数无效
    #[error("请求参数无效: {0}")]
    InvalidRequest(String),

    /// 配置相关错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 请求错误
    #[error("请求错误: {0}")]
    RequestError(String),

    /// UTF8转换错误
    #[error("UTF8转换错误: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// NanoAI 库的 Result 类型别名
pub type Result<T> = std::result::Result<T, NanoError>;

impl From<serde_json::Error> for NanoError {
    fn from(e: serde_json::Error) -> Self {
        NanoError::Json(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for NanoError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        NanoError::Utf8(e.utf8_error())
    }
}
