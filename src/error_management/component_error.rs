use std::fmt;

#[derive(Debug, Clone)]
pub enum ComponentError {}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {}
    }
}
