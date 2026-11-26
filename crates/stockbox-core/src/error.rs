use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("base de datos: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("credencial: {0}")]
    Password(#[from] password_hash::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Validation(String),
    #[error("no encontrado: {0}")]
    NotFound(String),
    #[error("conflicto: {0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
