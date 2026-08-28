use anyhow::{Context, Result};
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader};
use palette::{
	Clamp, FromColor, IntoColor, LinSrgb, LinSrgba, Srgb, Srgba, Xyz,
	cam16::{BakedParameters, Cam16, Cam16Jmh, Cam16UcsJab, Parameters, StaticWp, Surround},
	cast::ComponentsAs,
	white_point::D65,
};
use rayon::prelude::*;
use std::io::Cursor;
use tracing::debug;

// Discord CDN URLs carry size/quality as query params; strip and replace to control output
pub fn change_to_x128_url(url: &str) -> String {
	let base_url = url.split('?').next().unwrap_or(url);
	format!("{}?size=128&quality=lossless", base_url)
}

pub async fn get_image_from_url(url: &str) -> Result<DynamicImage> {
	let resp = reqwest::get(url)
		.await
		.context(format!("Failed to fetch image from URL: {}", url))?
		.bytes()
		.await
		.context(format!("Failed to get image bytes from URL: {}", url))?;

	let url_owned = url.to_string();
	// Image decoding is CPU-intensive; run off the async runtime to avoid blocking it
	tokio::task::spawn_blocking(move || {
		let img = ImageReader::new(Cursor::new(resp))
			.with_guessed_format()
			.context(format!(
				"Failed to guess image format from URL: {}",
				url_owned
			))?
			.decode()
			.context(format!("Failed to decode image from URL: {}", url_owned))?;
		Ok(img)
	})
	.await
	.context("spawn_blocking panicked")?
}

fn change_to_full_size_url(url: &str) -> String {
	let base_url = url.split('?').next().unwrap_or(url);
	format!("{}?size=4096&quality=lossless", base_url)
}

// NOTE: avatars are synthetic sRGB graphics, not photographs of a scene under some unknown
// illuminant, so no chromatic adaptation is applied. Estimating a white point from the
// brightest pixels used to read a bright yellow background as "white" and swing the whole
// image toward blue/purple. Input is treated as already D65-referenced.

pub fn make_params() -> BakedParameters<StaticWp<D65>, f32> {
	let mut p = Parameters::default_static_wp(64.0f32); // adapting_luminance
	p.background_luminance = 0.2;
	p.surround = Surround::Average;
	p.bake()
}

// The mosaic needs the *optical spatial average* of a tile -- what the eye integrates when the
// tile is shrunk down -- not the most prominent distinct object in it. A plain alpha-weighted
// mean in CAM16-UCS gives exactly that. The previous k-means pass filtered to "chromatic"
// pixels first (chroma > 8, 10 < J < 95), which threw away skin tones, pale highlights and
// near-white backgrounds; a tile that was 95% pale skin with a few dark green pixels came back
// dark green. Nothing is discarded here except fully transparent pixels.
pub fn calculate_tile_mean_cam16(linear_pixels: &[LinSrgba<f32>]) -> Option<Cam16UcsJab<f32>> {
	let params = make_params();

	// rayon fold/reduce keeps the per-pixel CAM16 forward transform parallel: a full-size
	// avatar is 4096x4096, so this runs over ~16M pixels.
	let (sum_j, sum_a, sum_b, total_weight) = linear_pixels
		.par_iter()
		.filter(|pixel| pixel.alpha >= 0.05)
		.map(|pixel| {
			let rgb = LinSrgb::new(
				pixel.red * pixel.alpha,
				pixel.green * pixel.alpha,
				pixel.blue * pixel.alpha,
			);

			let xyz: Xyz<D65, f32> = rgb.into_color();
			let ucs = Cam16UcsJab::from_color(Cam16::from_xyz(xyz, params));

			(
				ucs.lightness * pixel.alpha,
				ucs.a * pixel.alpha,
				ucs.b * pixel.alpha,
				pixel.alpha,
			)
		})
		.reduce(
			|| (0.0f32, 0.0f32, 0.0f32, 0.0f32),
			|a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
		);

	// Fully transparent image: nothing to average
	if total_weight < 1e-4 {
		return None;
	}

	Some(Cam16UcsJab {
		lightness: sum_j / total_weight,
		a: sum_a / total_weight,
		b: sum_b / total_weight,
	})
}

pub fn cam16ucs_to_srgb_u8(dominant: Cam16UcsJab<f32>) -> [u8; 3] {
	let params = make_params();
	let jmh = Cam16Jmh::from_color(dominant);
	let cam16: Cam16<f32> = jmh.into_full(params);
	let xyz: Xyz<D65, f32> = cam16.into_xyz(params);
	let linear: LinSrgb<f32> = xyz.into_color();
	let linear = Clamp::clamp(linear);
	let srgb_f32: Srgb<f32> = Srgb::from_linear(linear);
	let srgb_u8: Srgb<u8> = srgb_f32.into_format();
	[srgb_u8.red, srgb_u8.green, srgb_u8.blue]
}

pub fn cam16ucs_to_hex(r: u8, g: u8, b: u8) -> String {
	format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub async fn calculate_user_color_from_url(
	profile_picture_url: &str,
) -> Result<(String, Vec<u8>, Vec<u8>)> {
	let url = change_to_full_size_url(profile_picture_url);
	let img = get_image_from_url(&url).await?;

	tokio::task::spawn_blocking(move || {
		let img = img.to_rgba8();
		let raw: &[u8] = img.as_raw();
		let srgba_pixels: &[Srgba<u8>] = raw.components_as();
		let linear_pixels: Vec<LinSrgba<f32>> =
			srgba_pixels.par_iter().map(|p| p.into_linear()).collect();

		let mean = calculate_tile_mean_cam16(&linear_pixels)
			.ok_or_else(|| anyhow::anyhow!("image is fully transparent"))?;
		let cam16_str = format!("cam16;{};{};{}", mean.lightness, mean.a, mean.b);

		debug!("Calculated color: {}", cam16_str);

		let mut full_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut full_png_bytes).write_image(
			img.as_raw(),
			img.width(),
			img.height(),
			ExtendedColorType::Rgba8,
		)?;

		// Lanczos3 for the thumbnail: higher quality downscale than bilinear
		let thumb = image::imageops::resize(&img, 128, 128, image::imageops::FilterType::Lanczos3);
		let mut thumb_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut thumb_png_bytes).write_image(
			thumb.as_raw(),
			thumb.width(),
			thumb.height(),
			ExtendedColorType::Rgba8,
		)?;

		// Returns: hex color string, 128x128 thumbnail (for mosaic tiles), full-size PNG (for display)
		Ok((cam16_str, thumb_png_bytes, full_png_bytes))
	})
	.await
	.context("spawn_blocking panicked")?
}

#[cfg(test)]
mod tests {
	use super::*;
	use palette::Srgba;

	fn cam16_of(r: u8, g: u8, b: u8) -> Cam16UcsJab<f32> {
		let linear: LinSrgb<f32> = Srgb::new(r, g, b).into_linear();
		let xyz: Xyz<D65, f32> = linear.into_color();
		Cam16UcsJab::from_color(Cam16::from_xyz(xyz, make_params()))
	}

	fn lin(pixels: &[(u8, u8, u8, u8)]) -> Vec<LinSrgba<f32>> {
		pixels
			.iter()
			.map(|(r, g, b, a)| Srgba::new(*r, *g, *b, *a).into_linear())
			.collect()
	}

	#[test]
	fn solid_color_mean_equals_that_color() {
		let px = lin(&vec![(192u8, 128, 64, 255); 64]);
		let mean = calculate_tile_mean_cam16(&px).unwrap();

		let expected = cam16_of(192, 128, 64);
		assert!(
			(mean.lightness - expected.lightness).abs() < 0.01,
			"{mean:?}"
		);
		assert!((mean.a - expected.a).abs() < 0.01);
		assert!((mean.b - expected.b).abs() < 0.01);
	}

	#[test]
	fn pale_skin_majority_is_not_hijacked_by_a_few_dark_pixels() {
		// 95% pale skin (low chroma, high J -> the old is_chromatic filter dropped it),
		// 5% dark green. The mean must stay near the skin tone.
		let mut px = vec![(245u8, 214, 190, 255); 95];
		px.extend(vec![(10u8, 60, 20, 255); 5]);
		let mean = calculate_tile_mean_cam16(&lin(&px)).unwrap();

		let skin = cam16_of(245, 214, 190);
		let dark_green = cam16_of(10, 60, 20);
		let d_skin = (mean.lightness - skin.lightness).abs();
		let d_green = (mean.lightness - dark_green.lightness).abs();
		assert!(d_skin < d_green, "mean {mean:?} closer to green than skin");
	}

	#[test]
	fn fully_transparent_returns_none() {
		assert!(calculate_tile_mean_cam16(&lin(&vec![(255u8, 0, 0, 0); 16])).is_none());
	}

	#[test]
	fn pure_grayscale_no_longer_fails() {
		// Used to bail with "image is fully achromatic" -- every pixel failed is_chromatic.
		let px = lin(&vec![(128u8, 128, 128, 255); 32]);
		assert!(calculate_tile_mean_cam16(&px).is_some());
	}
}
