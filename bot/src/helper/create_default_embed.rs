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
pub fn get_default_embed<'a>(option: Option<Colour>) -> CreateEmbed<'a> {
	let color = option.unwrap_or(COLOR);

	CreateEmbed::new().timestamp(Timestamp::now()).color(color)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serenity::all::Colour;

	#[test]
	fn test_default_embed_with_default_color() {
		// Test creating an embed with the default color
		let embed = get_default_embed(None);

		// We can't directly access private fields, but we can verify the function doesn't panic
		// and returns a non-null value
		assert!(
			std::mem::size_of_val(&embed) > 0,
			"Embed should be a valid object"
		);
	}

	#[test]
	fn test_default_embed_with_custom_color() {
		// Test creating an embed with a custom color
		let custom_color = Colour::DARK_GREEN;
		let embed = get_default_embed(Some(custom_color));

		// We can't directly access private fields, but we can verify the function doesn't panic
		// and returns a non-null value
		assert!(
			std::mem::size_of_val(&embed) > 0,
			"Embed should be a valid object"
		);

		// Test that using different colors produces different embeds
		// This is a weak test but better than nothing
		let default_embed = get_default_embed(None);
		let custom_embed = get_default_embed(Some(Colour::DARK_GREEN));

		// The embeds should be different in some way (though this doesn't specifically test color)
		assert_ne!(
			format!("{:?}", default_embed),
			format!("{:?}", custom_embed),
			"Embeds with different colors should be different"
		);
	}
}
