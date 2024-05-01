use std::error::Error;

// Custom error type for directory operations
#[derive(Debug)]
pub(crate) enum DirectoryError {
    NotFound,
    InvalidPath,
    NotAFile,
    NotADirectory,
}

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DirectoryError::NotFound => write!(f, "File or directory not found"),
            DirectoryError::InvalidPath => write!(f, "Invalid path"),
            DirectoryError::NotAFile => write!(f, "Path is not a valid file"),
            DirectoryError::NotADirectory => write!(f, "Path is not a valid directory"),
        }
    }
}

impl Error for DirectoryError {}
