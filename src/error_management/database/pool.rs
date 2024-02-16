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