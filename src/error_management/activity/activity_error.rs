use crate::error_management::database::pool::CreatingPoolError;

#[derive(Debug, Clone)]
pub enum ActivityError {
    Pool(CreatingPoolError),
}

impl From<CreatingPoolError> for ActivityError {
    fn from(error: CreatingPoolError) -> Self {
        ActivityError::Pool(error)
    }
}