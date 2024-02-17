use crate::error_management::generic_error::GenericError;
use std::fmt;
use crate::error_management::api_request_error::ApiRequestError;
use crate::error_management::lang_error::LangError;

#[derive(Debug, Clone)]
pub enum CommandError {
    Generic(GenericError),
    NotNSFW(String),
    Lang(LangError),
    APIRequestError(ApiRequestError),
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

impl From<ApiRequestError> for CommandError {
    fn from(error: ApiRequestError) -> Self {
        CommandError::APIRequestError(error)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Generic(generic_error) => write!(f, "Generic error: {}", generic_error),
            CommandError::NotNSFW(not_nsfw) => write!(f, "Not NSFW: {}", not_nsfw),
            CommandError::Lang(lang_error) => write!(f, "Lang error: {}", lang_error),
            CommandError::APIRequestError(api_request_error) => write!(f, "API request error: {}", api_request_error),
        }
    }
}
