use std::fmt;

pub struct CreatingPoolError {
    message: String,
}

impl CreatingPoolError {
    pub fn new(message: String) -> Self {
        CreatingPoolError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for CreatingPoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Message: {}", self.message)
    }
}
