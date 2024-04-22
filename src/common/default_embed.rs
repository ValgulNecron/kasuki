use serenity::all::{Colour, CreateEmbed, Timestamp};

use crate::constant::COLOR;

pub fn get_default_embed(option: Option<Colour>) -> CreateEmbed {
    let color = option.unwrap_or(COLOR);
    CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}
