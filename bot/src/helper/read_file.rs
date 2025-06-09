use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use tracing::{debug, error, info, trace};

/// Reads a file from the filesystem and returns its contents as a string.
///
/// This utility function provides a simple way to read text files with proper
/// error handling and logging. It follows a two-step process:
/// 1. Open the file from the specified path
/// 2. Read the entire contents of the file into a string
///
/// # Parameters
///
/// * `file_path` - The path to the file to be read
///
/// # Returns
///
/// * `Result<String>` - The file contents as a string if successful, or an error
///   with context if any step fails
///
/// # Error Handling
///
/// The function provides detailed error context for debugging purposes:
/// - If the file cannot be opened, it returns an error with the file path
/// - If the file contents cannot be read, it returns an error with the file path
///
/// # Performance Considerations
///
/// This function reads the entire file into memory at once, which may not be
/// appropriate for very large files. For large files, consider using a streaming
/// approach or reading the file in chunks.
///
pub fn read_file_as_string(file_path: &str) -> Result<String> {
	trace!("Reading file as string: {}", file_path);

	// Step 1: Open the file
	// This can fail if the file doesn't exist, permissions are insufficient,
	// or there are other filesystem-related issues
	debug!("Opening file: {}", file_path);
	let mut file = match File::open(file_path) {
		Ok(f) => {
			debug!("Successfully opened file: {}", file_path);
			f
		},
		Err(e) => {
			error!("Failed to open file {}: {}", file_path, e);
			return Err::<String, anyhow::Error>(e.into()).with_context(|| format!("Failed to open file: {}", file_path));
		}
	};

	// Prepare a string to hold the file contents
	let mut string_data = String::new();

	// Step 2: Read the file contents into the string
	// This can fail if the file is not valid UTF-8 or if there are
	// I/O errors during reading
	debug!("Reading file contents: {}", file_path);
	match file.read_to_string(&mut string_data) {
		Ok(bytes) => {
			debug!("Successfully read {} bytes from file: {}", bytes, file_path);
		},
		Err(e) => {
			error!("Failed to read contents from file {}: {}", file_path, e);
			return Err::<String, anyhow::Error>(e.into()).with_context(|| format!("Failed to read file contents: {}", file_path));
		}
	}

	// Return the file contents as a string
	info!("Successfully read file as string: {}", file_path);
	Ok(string_data)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::io::Write;
	use tempfile::tempdir;

	#[test]
	fn test_read_existing_file() {
		// Test reading an existing file in the project
		let result = read_file_as_string("test.json");
		assert!(result.is_ok(), "Should successfully read an existing file");
	}

	#[test]
	fn test_read_nonexistent_file() {
		// Test reading a file that doesn't exist
		let result = read_file_as_string("nonexistent_file.txt");
		assert!(result.is_err(), "Should return an error for a nonexistent file");

		// Verify the error message contains the file path
		let err = result.unwrap_err().to_string();
		assert!(err.contains("nonexistent_file.txt"), "Error should contain the file path");
	}

	#[test]
	fn test_read_file_with_known_content() {
		// Create a temporary directory that will be automatically cleaned up
		let dir = tempdir().expect("Failed to create temporary directory");
		let file_path = dir.path().join("test_content.txt");

		// Create a file with known content
		let test_content = "Hello, world! This is a test file.";
		{
			let mut file = fs::File::create(&file_path).expect("Failed to create test file");
			file.write_all(test_content.as_bytes()).expect("Failed to write to test file");
		}

		// Read the file and verify the content
		let result = read_file_as_string(file_path.to_str().unwrap());
		assert!(result.is_ok(), "Should successfully read the test file");
		assert_eq!(result.unwrap(), test_content, "File content should match what was written");
	}
}
