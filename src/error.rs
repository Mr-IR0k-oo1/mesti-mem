use thiserror::Error;

#[derive(Error, Debug)]
pub enum MatisError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Executor error: {0}")]
    Executor(String),

    #[error("No active project")]
    NoActiveProject,
}
