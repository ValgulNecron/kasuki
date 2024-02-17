use crate::error_management::database_error::DatabaseError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum WebRequestError {
    Request(String),
    Parsing(String),
    NotFound(String),
    Decoding(String),
    IncorrectUrl(String),
    DatabaseError(DatabaseError),
    Header(String),
}

impl fmt::Display for WebRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebRequestError::Request(error) => write!(f, "Request error: {}", error),
            WebRequestError::Parsing(error) => write!(f, "Parsing error: {}", error),
            WebRequestError::NotFound(error) => write!(f, "Not found error: {}", error),
            WebRequestError::Decoding(error) => write!(f, "Decoding error: {}", error),
            WebRequestError::IncorrectUrl(error) => write!(f, "Incorrect url error: {}", error),
            WebRequestError::DatabaseError(database_error) => {
                write!(f, "Database error: {}", database_error)
            }
            WebRequestError::Header(error) => write!(f, "Header error: {}", error),
        }
    }
}

impl From<DatabaseError> for WebRequestError {
    fn from(error: DatabaseError) -> Self {
        WebRequestError::DatabaseError(error)
    }
}
