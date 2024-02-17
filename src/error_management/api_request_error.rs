use crate::error_management::file_error::FileError;

#[derive(Debug, Clone)]
pub enum ApiRequestError {
    Request(String),
    Parsing(String),
    NotFound(String),
    Decoding(String),
}