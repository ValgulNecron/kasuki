use image::DynamicImage;
use palette::color_difference::ImprovedDeltaE;
use palette::{IntoColor, Lab, Srgb};

#[derive(Clone, Debug)]
pub struct Color {
	pub cielab: Lab,
}

#[derive(Clone, Debug)]
pub struct ColorWithUrl {
	pub cielab: Lab,
	pub image: DynamicImage,
}

fn convert_hex_to_rgb(hex: &str) -> (u8, u8, u8) {
	(
		u8::from_str_radix(&hex[1..3], 16).unwrap_or_default(),
		u8::from_str_radix(&hex[3..5], 16).unwrap_or_default(),
		u8::from_str_radix(&hex[5..7], 16).unwrap_or_default(),
	)
}

/// Create color vector from tuples of `(hex_color, png_bytes)`.
pub fn create_color_vector(tuples: Vec<(String, Vec<u8>)>) -> Vec<ColorWithUrl> {
	tuples
		.into_iter()
		.filter_map(|(hex, png_bytes)| {
			let img = match image::load_from_memory(&png_bytes) {
				Ok(img) => img,
				Err(_) => return None,
			};

			let (r, g, b) = convert_hex_to_rgb(&hex);
			Some(get_color_with_url(img, r, g, b))
		})
		.collect()
}

pub fn find_closest_color_index(colors: &[ColorWithUrl], target: &Color) -> Option<usize> {
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

pub fn get_color_with_url(img: DynamicImage, r: u8, g: u8, b: u8) -> ColorWithUrl {
	let rgb_color = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
	let lab_color: Lab = rgb_color.into_color();

	ColorWithUrl {
		cielab: lab_color,
		image: img,
	}
}
