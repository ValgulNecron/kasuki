use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
	cynic_codegen::register_schema("anilist")
		.from_sdl_file("schemas/anilist.graphql")?
		.as_default()?;
	Ok(())
}

