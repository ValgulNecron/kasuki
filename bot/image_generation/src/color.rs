use anyhow::Context;
use image::RgbaImage;
use image::imageops::FilterType;
use palette::color_difference::ImprovedDeltaE;
use palette::{FromColor, IntoColor, Lab, LinSrgb, Srgb, Xyz};
use palette::cam16::{Cam16, Cam16UcsJab};
use palette::white_point::D65;
use crate::calculate::make_params;

#[derive(Clone, Debug)]
pub struct Color {
	pub cam16: Cam16UcsJab<f32>,
}

#[derive(Clone, Debug)]
pub struct ColorWithTile {
	pub cam16: Cam16UcsJab<f32>,
	pub tile: RgbaImage,
}


pub fn create_color_tile(color_string: &str, png_bytes: &[u8], tile_size: u32) -> Option<ColorWithTile> {
	let img = image::load_from_memory(png_bytes).ok()?;

	let cam16_ucs = match color_from_string(color_string) {
		Ok(a) => a,
		_ => return None,
	};

	// Triangle (bilinear) filter: fast and sufficient for small tile thumbnails
	let tile = image::imageops::resize(&img, tile_size, tile_size, FilterType::Triangle);

	Some(ColorWithTile {
		cam16: cam16_ucs.cam16,
		tile,
	})
}

// Finds the tile whose average color is perceptually closest to the target


pub fn find_closest_color_index(colors: &[ColorWithTile], target: &Color) -> Option<usize> {
	colors
		.iter()
		.enumerate()
		.min_by(|(_, a), (_, b)| {
			let da = cam16_delta_e(&a.cam16, &target.cam16);
			let db = cam16_delta_e(&b.cam16, &target.cam16);
			da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
		})
		.map(|(i, _)| i)
}

fn cam16_delta_e(a: &Cam16UcsJab<f32>, b: &Cam16UcsJab<f32>) -> f32 {
	let dj = a.lightness - b.lightness;
	let da = a.a - b.a;
	let db = a.b - b.b;
	(dj * dj + da * da + db * db).sqrt()
}

pub fn color_from_string(s: &str) -> anyhow::Result<Color> {
	if let Some(rest) = s.strip_prefix("cam16;") {
		let parts: Vec<&str> = rest.splitn(3, ';').collect();
		if parts.len() != 3 {
			anyhow::bail!("invalid cam16 color string: {s}");
		}
		let j: f32 = parts[0].parse().context("invalid J")?;
		let a: f32 = parts[1].parse().context("invalid a")?;
		let b: f32 = parts[2].parse().context("invalid b")?;
		Ok(Color {
			cam16: Cam16UcsJab { lightness: j, a, b },
		})
	} else if let Some(hex) = s.strip_prefix('#') {
		if hex.len() != 6 {
			anyhow::bail!("invalid hex color string: {s}");
		}
		let r = u8::from_str_radix(&hex[0..2], 16).context("invalid R")?;
		let g = u8::from_str_radix(&hex[2..4], 16).context("invalid G")?;
		let b = u8::from_str_radix(&hex[4..6], 16).context("invalid B")?;
		let params = make_params();
		let linear: LinSrgb<f32> = Srgb::new(r, g, b).into_linear();
		let xyz: Xyz<D65, f32> = linear.into_color();
		let cam16_ucs = Cam16UcsJab::from_color(Cam16::from_xyz(xyz, params));
		Ok(Color {cam16: cam16_ucs})
	} else {
		anyhow::bail!("unknown color format: {s}")
	}
}
