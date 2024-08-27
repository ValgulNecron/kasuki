use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn read_file_as_string(file_path: &str) -> Result<String, Box<dyn Error>> {
    // Open the JSON file and handle any potential errors
    let mut file = File::open(file_path)?;

    // Initialize a new String to hold the JSON data
    let mut string_data = String::new();

    // Read the JSON file into the String and handle any potential errors
    file.read_to_string(&mut string_data)?;

    Ok(string_data)
}
