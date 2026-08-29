pub mod character;
pub mod game;
pub mod producer;
pub mod staff;
pub mod stats;
pub mod user;

use crate::command::embed_content::{EmbedContent, EmbedsContents};
use shared::service::types::DisplayField;

fn build_vn_embed<'a>(
	title: String, description: Option<String>, fields: Vec<DisplayField>, url: String,
	image_url: Option<String>,
) -> EmbedsContents<'a> {
	let mut embed_content = EmbedContent::new(title)
		.description(description.unwrap_or_default())
		.fields(fields)
		.url(url);
	if let Some(img) = image_url {
		embed_content = embed_content.images_url(img);
	}
	EmbedsContents::new(vec![embed_content])
}
