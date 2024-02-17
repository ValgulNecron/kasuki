use crate::error_management::file_error::FileError;
use crate::error_management::generic_error::GenericError;
use crate::error_management::lang_error::LangError;
use crate::error_management::web_request_error::WebRequestError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum CommandError {
    Generic(GenericError),
    NotNSFW(String),
    Lang(LangError),
    WebRequestError(WebRequestError),
    File(FileError),
}

impl From<GenericError> for CommandError {
    fn from(error: GenericError) -> Self {
        CommandError::Generic(error)
    }
}

impl From<LangError> for CommandError {
    fn from(error: LangError) -> Self {
        CommandError::Lang(error)
    }
}

impl From<WebRequestError> for CommandError {
    fn from(error: WebRequestError) -> Self {
        CommandError::WebRequestError(error)
    }
}

impl From<FileError> for CommandError {
    fn from(error: FileError) -> Self {
        CommandError::File(error)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Generic(generic_error) => write!(f, "Generic error: {}", generic_error),
            CommandError::NotNSFW(not_nsfw) => write!(f, "Not NSFW: {}", not_nsfw),
            CommandError::Lang(lang_error) => write!(f, "Lang error: {}", lang_error),
            CommandError::WebRequestError(api_request_error) => {
                write!(f, "API request error: {}", api_request_error)
            }
            CommandError::File(file_error) => write!(f, "File error: {}", file_error),
        }
    }
}
