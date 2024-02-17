use std::fmt;

pub enum FileError {
    Writing(String),
    Reading(String),
    Creating(String),
    Decoding(String),
    NotFound(String),
    Parsing(String),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::Writing(writing) => write!(f, "Writing file error: {}", writing),
            FileError::Reading(reading) => write!(f, "Reading file error: {}", reading),
            FileError::Creating(creating) => write!(f, "Creating file error: {}", creating),
            FileError::Decoding(decoding) => write!(f, "Decoding file error: {}", decoding),
            FileError::NotFound(not_found) => write!(f, "File not found error: {}", not_found),
            FileError::Parsing(parsing) => write!(f, "Parsing file error: {}", parsing),
        }
    }
}
