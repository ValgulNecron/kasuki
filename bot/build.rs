use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	cynic_codegen::register_schema("anilist")
		.from_sdl_file("schemas/anilist.graphql")?
		.as_default()?;

	Ok(())
}
