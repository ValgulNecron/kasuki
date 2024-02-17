use crate::error_management::file_error::FileError;
use crate::error_management::generic_error::GenericError;
use crate::error_management::image_error::ImageError;
use crate::error_management::web_request_error::WebRequestError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DifferedCommandError {
    Generic(GenericError),
    WebRequestError(WebRequestError),
    FileError(FileError),
    ImageError(ImageError),
}

impl From<GenericError> for DifferedCommandError {
    fn from(error: GenericError) -> Self {
        DifferedCommandError::Generic(error)
    }
}

impl From<WebRequestError> for DifferedCommandError {
    fn from(error: WebRequestError) -> Self {
        DifferedCommandError::WebRequestError(error)
    }
}

impl From<FileError> for DifferedCommandError {
    fn from(error: FileError) -> Self {
        DifferedCommandError::FileError(error)
    }
}

impl From<ImageError> for DifferedCommandError {
    fn from(error: ImageError) -> Self {
        DifferedCommandError::ImageError(error)
    }
}

impl fmt::Display for DifferedCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DifferedCommandError::Generic(generic_error) => {
                write!(f, "Generic error: {}", generic_error)
            }
            DifferedCommandError::WebRequestError(api_request_error) => {
                write!(f, "API request error: {}", api_request_error)
            }
            DifferedCommandError::FileError(file_error) => {
                write!(f, "File error: {}", file_error)
            }
            DifferedCommandError::ImageError(image_error) => {
                write!(f, "Image error: {}", image_error)
            }
        }
    }
}
