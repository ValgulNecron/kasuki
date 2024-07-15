use crate::helper::error_management::error_enum::UnknownResponseError;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn read_file_as_string(file_path: &str) -> Result<String, Box<dyn Error>> {
    // Open the JSON file and handle any potential errors
    let mut file =
        File::open(file_path).map_err(|e| UnknownResponseError::File(format!("{:#?}", e)))?;

    // Initialize a new String to hold the JSON data
    let mut string_data = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut string_data)
        .map_err(|e| UnknownResponseError::File(format!("{:#?}", e)))?;

    Ok(string_data)
}
