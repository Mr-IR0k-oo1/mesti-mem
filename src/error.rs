use thiserror::Error;

#[derive(Error, Debug)]
pub enum MatisError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Executor: {0}")]
    Executor(String),
    #[error("Validation: {0}")]
    Validation(String),
}
