use std::fmt;

#[derive(Debug, Clone)]
pub enum DatabaseError {
    Insert(String),
    Create(String),
    Select(String),
    CreatePool(String),
    Alter(String),
    Update(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::insert(error) => write!(f, "Insert error: {}", error),
            DatabaseError::create(error) => write!(f, "Create error: {}", error),
            DatabaseError::select(error) => write!(f, "Select error: {}", error),
            DatabaseError::create_pool(error) => write!(f, "Create pool error: {}", error),
            DatabaseError::alter(error) => write!(f, "Alter error: {}", error),
            DatabaseError::update(error) => write!(f, "Update error: {}", error),
        }
    }
}