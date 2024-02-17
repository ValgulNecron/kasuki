use std::fmt;
use crate::error_management::file_error::FileError;

#[derive(Debug, Clone)]
pub enum WebHookError {
    Creating(String),
    Sending(String),
    Parsing(String),
    NotFound(String),
    Editing(String),
}

impl fmt::Display for WebHookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebHookError::Creating(creating) => write!(f, "Creating webhook error: {}", creating),
            WebHookError::Sending(sending) => write!(f, "Sending webhook error: {}", sending),
            WebHookError::Parsing(parsing) => write!(f, "Parsing webhook error: {}", parsing),
            WebHookError::NotFound(not_found) => write!(f, "Webhook not found error: {}", not_found),
            WebHookError::Editing(editing) => write!(f, "Editing webhook error: {}", editing),
        }
    }
}