use crate::constant::COLOR;
use serenity::all::{Colour, CreateEmbed, Timestamp};

pub fn get_default_embed<'a>(
	option: Option<u32>, user_colour: &Option<Colour>,
) -> CreateEmbed<'a> {
	let color = option
		.map(Colour::new)
		.unwrap_or(user_colour.unwrap_or(COLOR));

	CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}
