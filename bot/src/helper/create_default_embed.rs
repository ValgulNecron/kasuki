use crate::constant::COLOR;
use serenity::all::{Colour, CreateEmbed, Timestamp};

pub fn get_default_embed<'a>(option: Option<Colour>) -> CreateEmbed<'a> {
	let color = option.unwrap_or(COLOR);

	CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}
