use std::fmt;
use crate::error_management::database_error::DatabaseError;
use crate::error_management::lang_error::LangError;
use crate::error_management::webhook_error::WebHookError;

#[derive(Debug, Clone)]
pub enum ActivityError {
    Lang(LangError),
    DatabaseError(DatabaseError),
    WebhookError(WebHookError),
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

impl From<WebHookError> for ActivityError {
    fn from(error: WebHookError) -> Self {
        ActivityError::WebhookError(error)
    }
}

impl fmt::Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActivityError::Pool(pool_error) => write!(f, "Pool error: {}", pool_error),
            ActivityError::Lang(lang_error) => write!(f, "Lang error: {}", lang_error),
            ActivityError::DatabaseError(database_error) => write!(f, "Database error: {}", database_error),
            ActivityError::WebhookError(webhook_error) => write!(f, "Webhook error: {}", webhook_error),
        }
    }
}
