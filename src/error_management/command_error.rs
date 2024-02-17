use crate::error_management::generic_error::GenericError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum CommandError {
    Generic(GenericError),
    NotNSFW(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Generic(generic_error) => write!(f, "Generic error: {}", generic_error),
            CommandError::NotNSFW(not_nsfw) => write!(f, "Not NSFW: {}", not_nsfw),
        }
    }
}
