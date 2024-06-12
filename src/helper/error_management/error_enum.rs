use std::fmt;

// AppError is a struct that represents an application error
#[derive(Debug, Clone)]
pub struct AppError {
    // message is a String that contains the error message
    pub message: String,
    // error_type is an ErrorType that represents the type of the error
    pub error_type: ErrorType,
    // error_response_type is an ErrorResponseType that represents the type of the error response
    pub error_response_type: ErrorResponseType,
}

impl AppError {
    /// `new` is a function that creates a new `AppError`.
    /// It takes a `message`, `error_type`, and `error_response_type` as parameters.
    /// `message` is a String, `error_type` is an ErrorType, and `error_response_type` is an ErrorResponseType.
    /// It returns a new `AppError`.
    ///
    /// # Arguments
    ///
    /// * `message` - A String that represents the error message.
    /// * `error_type` - An ErrorType that represents the type of the error.
    /// * `error_response_type` - An ErrorResponseType that represents the type of the error response.
    ///
    /// # Returns
    ///
    /// * `AppError` - A new `AppError`.
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
            "Error type: {}. Error message: {}",
            self.error_type, self.message
        )
    }
}

// ErrorType is an enum that represents the type of an error
#[derive(Debug, Clone)]
pub enum ErrorType {
    Command,
    Database,
    File,
    Option,
    WebRequest,
    Webhook,
    Image,
    Module,
    Component,
    Logging,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::Command => write!(f, "Command"),
            ErrorType::Database => write!(f, "Database"),
            ErrorType::File => write!(f, "File"),
            ErrorType::Option => write!(f, "Option"),
            ErrorType::WebRequest => write!(f, "WebRequest"),
            ErrorType::Webhook => write!(f, "Webhook"),
            ErrorType::Image => write!(f, "Image"),
            ErrorType::Module => write!(f, "Module"),
            ErrorType::Component => write!(f, "Component"),
            ErrorType::Logging => write!(f, "Logging"),
        }
    }
}

// ErrorResponseType is an enum that represents the type of an error response
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
