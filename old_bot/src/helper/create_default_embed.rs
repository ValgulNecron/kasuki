use crate::constant::COLOR;
use serenity::all::{Colour, CreateEmbed, Timestamp};

/// Creates a default embed with the current timestamp and specified color.
///
/// # Arguments
///
/// * `option` - An optional color for the embed. If None, uses the default color from constants.
///
/// # Returns
///
/// A new CreateEmbed instance with timestamp and color set.
///
pub fn get_default_embed<'a>(
    option: Option<Colour>, user_colour: &Option<Colour>,
) -> CreateEmbed<'a> {
    let color = option.unwrap_or(user_colour.unwrap_or(COLOR));

    CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}
