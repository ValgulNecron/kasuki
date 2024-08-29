use serenity::all::{Colour, CreateEmbed, Timestamp};

use crate::constant::COLOR;

/// Creates a default embed with a timestamp and a color.
///
/// This function creates a new embed with the current timestamp.
/// It then sets the color of the embed to the color specified in the `option` parameter.
/// If `option` is `None`, it sets the color to the default color specified in the `COLOR` constant.
///
/// # Arguments
///
/// * `option` - An `Option` that may contain a `Colour` to be used for the embed. If `None`, the default `COLOR` is used.
///
/// # Returns
///
/// * A `CreateEmbed` instance with the current timestamp and the specified color.
pub fn get_default_embed(option: Option<Colour>) -> CreateEmbed {
    let color = option.unwrap_or(COLOR);
    CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}
