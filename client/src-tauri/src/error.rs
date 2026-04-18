//! 应用错误类型定义
//!
//! 本模块定义了应用中所有可能的错误类型，并提供统一的错误处理机制。

use thiserror::Error;

/// 应用错误枚举
///
/// 包含所有可能的应用错误类型，使用 `thiserror` 派生错误消息格式化。
#[derive(Error, Debug)]
pub enum AppError {
    /// 未知错误
    #[error("Unknown error")]
    Unknown,

    /// 缺少应用句柄
    #[error("Missing app handle")]
    MissingAppHandle,

    /// Store 打开错误
    #[error("Store open error: {0}")]
    StoreOpen(String),

    /// Store 保存错误
    #[error("Store save error: {0}")]
    StoreSave(String),

    /// 认证缺少请求头
    #[error("Missing authentication header 'x-openinbrowser-auth'")]
    AuthMissingHeader,

    /// JWT 认证无效
    #[error("Invalid JWT authentication: {0}")]
    AuthJwtInvalid(String),

    /// HTTP 服务器绑定错误
    #[allow(dead_code)]
    #[error("HTTP server bind error: {0}")]
    HttpBind(String),

    /// I/O 错误
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

/// 实现 From<String> 以便更容易地转换字符串错误
impl From<String> for AppError {
    fn from(_error: String) -> Self {
        AppError::Unknown // 可以根据需要添加更具体的错误类型
    }
}
