use crate::error_management::file_error::FileError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LangError {
    File(FileError),
    NotFound(),
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LangError::File(file_error) => write!(f, "File error: {}", file_error),
            LangError::NotFound() => write!(f, "Language not found error"),
        }
    }
}

impl From<FileError> for LangError {
    fn from(error: FileError) -> Self {
        LangError::File(error)
    }
}
