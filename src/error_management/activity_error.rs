use crate::error_management::database_error::DatabaseError;
use crate::error_management::file_error::FileError;
use crate::error_management::generic_error::GenericError;
use crate::error_management::lang_error::LangError;
use crate::error_management::web_request_error::WebRequestError;
use crate::error_management::webhook_error::WebhookError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ActivityError {
    Lang(LangError),
    DatabaseError(DatabaseError),
    WebhookError(WebhookError),
    WebRequestError(WebRequestError),
    FileError(FileError),
    Generic(GenericError),
}

impl From<LangError> for ActivityError {
    fn from(error: LangError) -> Self {
        ActivityError::Lang(error)
    }
}

impl From<DatabaseError> for ActivityError {
    fn from(error: DatabaseError) -> Self {
        ActivityError::DatabaseError(error)
    }
}

impl From<WebhookError> for ActivityError {
    fn from(error: WebhookError) -> Self {
        ActivityError::WebhookError(error)
    }
}

impl From<WebRequestError> for ActivityError {
    fn from(error: WebRequestError) -> Self {
        ActivityError::WebRequestError(error)
    }
}

impl From<FileError> for ActivityError {
    fn from(error: FileError) -> Self {
        ActivityError::FileError(error)
    }
}

impl From<GenericError> for ActivityError {
    fn from(error: GenericError) -> Self {
        ActivityError::Generic(error)
    }
}

impl fmt::Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActivityError::Lang(lang_error) => write!(f, "Lang error: {}", lang_error),
            ActivityError::DatabaseError(database_error) => {
                write!(f, "Database error: {}", database_error)
            }
            ActivityError::WebhookError(webhook_error) => {
                write!(f, "Webhook error: {}", webhook_error)
            }
            ActivityError::WebRequestError(api_error) => write!(f, "Api error: {}", api_error),
            ActivityError::FileError(file_error) => write!(f, "File error: {}", file_error),
            ActivityError::Generic(generic_error) => write!(f, "Generic error: {}", generic_error),
        }
    }
}
