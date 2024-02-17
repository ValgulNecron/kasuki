use std::fmt;

#[derive(Debug, Clone)]
pub enum WebhookError {
    Creating(String),
    Sending(String),
    Parsing(String),
    NotFound(String),
    Editing(String),
}

impl fmt::Display for WebhookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebhookError::Creating(creating) => write!(f, "Creating webhook error: {}", creating),
            WebhookError::Sending(sending) => write!(f, "Sending webhook error: {}", sending),
            WebhookError::Parsing(parsing) => write!(f, "Parsing webhook error: {}", parsing),
            WebhookError::NotFound(not_found) => write!(f, "Webhook not found error: {}", not_found),
            WebhookError::Editing(editing) => write!(f, "Editing webhook error: {}", editing),
        }
    }
}