use std::fmt;

#[derive(Debug, Clone)]
pub enum CommandError {
}

impl fmt::Display for CommandError  {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {

        }
    }
}