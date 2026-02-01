use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use tracing::{debug, error, info, trace};

pub fn read_file_as_string(file_path: &str) -> Result<String> {
	trace!("Reading file as string: {}", file_path);

	debug!("Opening file: {}", file_path);
	let mut file = match File::open(file_path) {
		Ok(f) => {
			debug!("Successfully opened file: {}", file_path);
			f
		},
		Err(e) => {
			error!("Failed to open file {}: {}", file_path, e);
			return Err::<String, anyhow::Error>(e.into())
				.with_context(|| format!("Failed to open file: {}", file_path));
		},
	};

	let mut string_data = String::new();

	debug!("Reading file contents: {}", file_path);
	match file.read_to_string(&mut string_data) {
		Ok(bytes) => {
			debug!("Successfully read {} bytes from file: {}", bytes, file_path);
		},
		Err(e) => {
			error!("Failed to read contents from file {}: {}", file_path, e);
			return Err::<String, anyhow::Error>(e.into())
				.with_context(|| format!("Failed to read file contents: {}", file_path));
		},
	}

	info!("Successfully read file as string: {}", file_path);
	Ok(string_data)
}
