use crate::error_management::database::pool_error::CreatingPoolError;
use crate::error_management::lang::lang_error::LangError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ActivityError {
    Pool(CreatingPoolError),
    Lang(LangError),
}

impl From<CreatingPoolError> for ActivityError {
    fn from(error: CreatingPoolError) -> Self {
        ActivityError::Pool(error)
    }
}

impl From<LangError> for ActivityError {
    fn from(error: LangError) -> Self {
        ActivityError::Lang(error)
    }
}
impl fmt::Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActivityError::Pool(pool_error) => write!(f, "Pool error: {}", pool_error),
            ActivityError::Lang(lang_error) => write!(f, "Lang error: {}", lang_error),
        }
    }
}
