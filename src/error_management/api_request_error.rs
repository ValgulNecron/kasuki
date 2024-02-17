use std::fmt;

#[derive(Debug, Clone)]
pub enum ApiRequestError {
    Request(String),
    Parsing(String),
    NotFound(String),
    Decoding(String),
}

impl fmt::Display for ApiRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiRequestError::Request(error) => write!(f, "Request error: {}", error),
            ApiRequestError::Parsing(error) => write!(f, "Parsing error: {}", error),
            ApiRequestError::NotFound(error) => write!(f, "Not found error: {}", error),
            ApiRequestError::Decoding(error) => write!(f, "Decoding error: {}", error),
        }
    }
}