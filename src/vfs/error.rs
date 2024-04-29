use std::error::Error;

// Custom error type for directory operations
#[derive(Debug)]
pub(crate) enum DirectoryError {
    NotFound,
    InvalidPath,
}

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DirectoryError::NotFound => write!(f, "File or directory not found"),
            DirectoryError::InvalidPath => write!(f, "Invalid path"),
        }
    }
}

impl Error for DirectoryError {}
