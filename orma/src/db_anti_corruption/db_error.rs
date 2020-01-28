use std::error::Error;

/// Errors are mapped to this structure
#[derive(Debug)]
pub struct DbError {
    /// Error description
    pub description: String,
    /// Optional error cause
    pub cause: Option<Box<dyn Error + Sync + Send>>,
}

impl DbError {
    /// Simple constructor
    pub fn new(description: &str, cause: Option<Box<dyn Error + Sync + Send>>) -> Self {
        Self {
            description: description.to_owned(),
            cause,
        }
    }
}

impl From<tokio_postgres::Error> for DbError {
    fn from(error: tokio_postgres::Error) -> Self {
        Self::new(&error.to_string(), Some(Box::new(error)))
    }
}

impl From<serde_json::Error> for DbError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(&error.to_string(), Some(Box::new(error)))
    }
}
