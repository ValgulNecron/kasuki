use crate::error_management::generic_error::GenericError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DifferedCommandError {
    Generic(GenericError),
}

impl fmt::Display for DifferedCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DifferedCommandError::Generic(generic_error) => {
                write!(f, "Generic error: {}", generic_error)
            }
        }
    }
}
