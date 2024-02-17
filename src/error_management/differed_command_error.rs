use crate::error_management::generic_error::GenericError;
use std::fmt;
use crate::error_management::api_request_error::ApiRequestError;

#[derive(Debug, Clone)]
pub enum DifferedCommandError {
    Generic(GenericError),
    APIRequestError(ApiRequestError),
}

impl From<GenericError> for DifferedCommandError {
    fn from(error: GenericError) -> Self {
        DifferedCommandError::Generic(error)
    }
}

impl From<ApiRequestError> for DifferedCommandError {
    fn from(error: ApiRequestError) -> Self {
        DifferedCommandError::APIRequestError(error)
    }
}

impl fmt::Display for DifferedCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DifferedCommandError::Generic(generic_error) => {
                write!(f, "Generic error: {}", generic_error)
            }
            DifferedCommandError::APIRequestError(api_request_error) => {
                write!(f, "API request error: {}", api_request_error)
            }
        }
    }
}
