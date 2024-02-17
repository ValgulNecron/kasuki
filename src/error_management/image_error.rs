use std::fmt;

#[derive(Debug, Clone)]
pub enum ImageError {
    CreateImage(String),
    ReadImage(String),
    WriteImage(String),
    DecodeImage(String),
    EncodeImage(String),
    IncorrectImage(String),
    ImageNotFound(String),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImageError::CreateImage(error) => write!(f, "Create image error: {}", error),
            ImageError::ReadImage(error) => write!(f, "Read image error: {}", error),
            ImageError::WriteImage(error) => write!(f, "Write image error: {}", error),
            ImageError::DecodeImage(error) => write!(f, "Decode image error: {}", error),
            ImageError::EncodeImage(error) => write!(f, "Encode image error: {}", error),
            ImageError::IncorrectImage(error) => write!(f, "Incorrect image error: {}", error),
            ImageError::ImageNotFound(error) => write!(f, "Image not found error: {}", error),
        }
    }
}
