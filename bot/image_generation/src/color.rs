use image::imageops::FilterType;
use image::RgbaImage;
use palette::color_difference::ImprovedDeltaE;
use palette::{IntoColor, Lab, Srgb};

#[derive(Clone, Debug)]
pub struct Color {
	pub cielab: Lab,
}

#[derive(Clone, Debug)]
pub struct ColorWithTile {
	pub cielab: Lab,
	pub tile: RgbaImage,
}

fn convert_hex_to_rgb(hex: &str) -> (u8, u8, u8) {
	(
		u8::from_str_radix(&hex[1..3], 16).unwrap_or_default(),
		u8::from_str_radix(&hex[3..5], 16).unwrap_or_default(),
		u8::from_str_radix(&hex[5..7], 16).unwrap_or_default(),
	)
}

pub fn create_color_tile(hex: &str, png_bytes: &[u8], tile_size: u32) -> Option<ColorWithTile> {
	let img = image::load_from_memory(png_bytes).ok()?;

	let (r, g, b) = convert_hex_to_rgb(hex);
	let rgb_color = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
	let lab_color: Lab = rgb_color.into_color();

	let tile = image::imageops::resize(&img, tile_size, tile_size, FilterType::Triangle);

	Some(ColorWithTile {
		cielab: lab_color,
		tile,
	})
}

pub fn find_closest_color_index(colors: &[ColorWithTile], target: &Color) -> Option<usize> {
	colors
		.iter()
		.enumerate()
		.min_by(|(_, a), (_, b)| {
			let delta_e_a = a.cielab.improved_delta_e(target.cielab);
			let delta_e_b = b.cielab.improved_delta_e(target.cielab);
			delta_e_a
				.partial_cmp(&delta_e_b)
				.unwrap_or(std::cmp::Ordering::Equal)
		})
		.map(|(i, _)| i)
}
