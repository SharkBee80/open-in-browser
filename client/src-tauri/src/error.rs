use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[allow(dead_code)]
    #[error("Unknown error")]
    Unknown,
    #[error("Missing app handle")]
    MissingAppHandle,
    #[error("Store open error: {0}")]
    StoreOpen(String),
    #[error("Store save error: {0}")]
    StoreSave(String),
    #[error("Config lock poisoned")]
    ConfigLockPoisoned,
    #[error("Auth missing header 'x-openinbrowser-auth'")]
    AuthMissingHeader,
    #[error("Auth JWT invalid: {0}")]
    AuthJwtInvalid(String),
    #[error("HTTP bind error: {0}")]
    HttpBind(String),
    // #[error("I/O error: {0}")]
    // IoError(#[from] std::io::Error),
    // #[error("JSON error: {0}")]
    // JsonError(#[from] serde_json::Error),
}
