use std::fmt;

#[derive(Debug, Clone)]
pub enum GenericError {
    OptionError(String),
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericError::OptionError(option_error) => write!(f, "Option error: {}", option_error),
        }
    }
}