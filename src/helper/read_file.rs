use std::fs::File;
use std::io::Read;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub fn read_file_as_string(file_path: &str) -> Result<String, AppError> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open(file_path).map_err(|e| {
        AppError::new(
            format!("File add_activity.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    // Initialize a new String to hold the JSON data
    let mut string_data = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut string_data).map_err(|e| {
        AppError::new(
            format!("File add_activity.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    Ok(string_data)
}
