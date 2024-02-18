use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: String,
    pub error_type: ErrorType,
    pub error_response_type: ErrorResponseType,
}

impl AppError {
    pub fn new(
        message: String,
        error_type: ErrorType,
        error_response_type: ErrorResponseType,
    ) -> Self {
        Self {
            message,
            error_type,
            error_response_type,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error type: {}. Error message: {}, Error response type: {}",
            self.error_type, self.message, self.error_response_type
        )
    }
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Activity,
    Command,
    Database,
    File,
    Generic,
    Option,
    WebRequest,
    Webhook,
    Language,
    Image,
    Module,
    Component,
    Logging,
    NewMember,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::Activity => write!(f, "Activity"),
            ErrorType::Command => write!(f, "Command"),
            ErrorType::Database => write!(f, "Database"),
            ErrorType::File => write!(f, "File"),
            ErrorType::Generic => write!(f, "Generic"),
            ErrorType::Option => write!(f, "Option"),
            ErrorType::WebRequest => write!(f, "WebRequest"),
            ErrorType::Webhook => write!(f, "Webhook"),
            ErrorType::Language => write!(f, "Language"),
            ErrorType::Image => write!(f, "Image"),
            ErrorType::Module => write!(f, "Module"),
            ErrorType::Component => write!(f, "Component"),
            ErrorType::Logging => write!(f, "Logging"),
            ErrorType::NewMember => write!(f, "NewMember"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorResponseType {
    Message,
    Followup,
    Unknown,
    None,
}

impl fmt::Display for ErrorResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorResponseType::Message => write!(f, "Message"),
            ErrorResponseType::Followup => write!(f, "Followup"),
            ErrorResponseType::Unknown => write!(f, "Unknown"),
            ErrorResponseType::None => write!(f, "None"),
        }
    }
}
