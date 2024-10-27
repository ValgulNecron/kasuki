use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub fn read_file_as_string(file_path: &str) -> Result<String> {
	let mut file = File::open(file_path)?;

	let mut string_data = String::new();

	file.read_to_string(&mut string_data)?;

	Ok(string_data)
}
