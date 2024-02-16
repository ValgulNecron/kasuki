use std::fmt;
use crate::error_management::file::file_error::FileError;
use crate::error_management::lang::lang_error::LangError;
#[derive(Debug, Clone)]
pub enum DatabaseError {
    Insert(String),
    Create(String),
}
impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::insert(error) => write!(f, "Insert error: {}", error),
            DatabaseError::create(error) => write!(f, "Create error: {}", error),
        }
    }
}