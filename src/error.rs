use thiserror::Error;

#[derive(Error, Debug)]
pub enum CartridgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Catalog error: {0}")]
    Catalog(String),

    #[error("AppImage inspection error: {0}")]
    Inspection(String),

    #[error("Extraction error: {0}")]
    Extraction(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Application '{0}' not found")]
    NotFound(String),

    #[error("Installation error: {0}")]
    Install(String),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Sandboxing error: {0}")]
    Sandbox(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CartridgeError>;
