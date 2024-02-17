use std::fmt;

#[derive(Debug, Clone)]
pub enum GenericError {
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {

        }
    }
}