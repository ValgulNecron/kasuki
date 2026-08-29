use anyhow::{Context, Result};
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader, RgbaImage};
use palette::{
	FromColor, IntoColor, LinSrgb, Srgb, Xyz,
	cam16::{BakedParameters, Cam16, Cam16UcsJab, Parameters, StaticWp, Surround},
	white_point::D65,
};
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

/// Version tag for the stored color string. Bump it whenever the descriptor algorithm changes:
/// the freshness window in handle_calculate_user_color skips records younger than 7 days, so
/// without a version bump a full recalculation right after a deploy silently no-ops and the
/// mosaic keeps matching on values from the previous algorithm.
pub const COLOR_STRING_PREFIX: &str = "cam16v3;";

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

// The mosaic needs the color an image *displays as* once it is shrunk down to tile size.
// Every resampler in the display chain (Discord's thumbnailer, browsers, image::imageops)
// averages gamma-encoded sRGB values, so the display-accurate mean is the plain average of
// the raw channel values -- exactly what a downscale to 1x1 produces. Averaging per-pixel
// CAM16-UCS coordinates instead ("average of appearances", the previous approach)
// underestimates lightness and chroma by an amount that grows with the tile's internal
// contrast, so every tile carried a different error and nearest-neighbor ranking on large
// palettes was effectively scrambled. The perceptual space is still used for matching: the
// single mean color is converted to CAM16-UCS afterwards, in srgb_to_cam16ucs.
//
// Alpha is resolved by compositing over WHITE, everywhere (this mean, the pasted tile, the
// guild-icon target): the mosaic is defined as an opaque image on a white ground. Many
// avatars are cutout PNGs with real transparency; weighting them by alpha instead described
// them by their visible subject's color, while the placed tile actually displays mostly
// white -- which is why white regions of the source rendered as gray.
pub fn mean_displayed_srgb(img: &RgbaImage) -> Option<Srgb<f32>> {
	let num_pixels = img.width() as f64 * img.height() as f64;
	if num_pixels == 0.0 {
		return None;
	}

	let (sum_r, sum_g, sum_b) = img
		.pixels()
		.fold((0.0f64, 0.0f64, 0.0f64), |(r, g, b), px| {
			let alpha = px[3] as f64 / 255.0;
			let white = 255.0 * (1.0 - alpha);
			(
				r + px[0] as f64 * alpha + white,
				g + px[1] as f64 * alpha + white,
				b + px[2] as f64 * alpha + white,
			)
		});

	let scale = num_pixels * 255.0;
	Some(Srgb::new(
		(sum_r / scale) as f32,
		(sum_g / scale) as f32,
		(sum_b / scale) as f32,
	))
}

// Flattens transparency against a white ground, in gamma space -- the same way a browser or
// Discord shows the PNG on a white page. Done before any resize: resizing straight-alpha
// RGBA first would bleed the hidden colors of transparent pixels into the edges.
pub fn composite_over_white(img: &RgbaImage) -> RgbaImage {
	let mut out = img.clone();
	for px in out.pixels_mut() {
		let alpha = px[3] as u32;
		if alpha == 255 {
			continue;
		}
		let inverse = 255 - alpha;
		px[0] = ((px[0] as u32 * alpha + 255 * inverse) / 255) as u8;
		px[1] = ((px[1] as u32 * alpha + 255 * inverse) / 255) as u8;
		px[2] = ((px[2] as u32 * alpha + 255 * inverse) / 255) as u8;
		px[3] = 255;
	}
	out
}

pub fn srgb_to_cam16ucs(
	srgb: Srgb<f32>, params: BakedParameters<StaticWp<D65>, f32>,
) -> Cam16UcsJab<f32> {
	let linear: LinSrgb<f32> = srgb.into_linear();
	let xyz: Xyz<D65, f32> = linear.into_color();
	Cam16UcsJab::from_color(Cam16::from_xyz(xyz, params))
}

pub async fn calculate_user_color_from_url(
	profile_picture_url: &str,
) -> Result<(String, Vec<u8>, Vec<u8>)> {
	let url = change_to_full_size_url(profile_picture_url);
	let img = get_image_from_url(&url).await?;

	tokio::task::spawn_blocking(move || {
		let img = img.to_rgba8();

		// Lanczos3 for the thumbnail: higher quality downscale than bilinear
		let thumb = image::imageops::resize(&img, 128, 128, image::imageops::FilterType::Lanczos3);

		// Computed once here, from the same thumbnail the mosaic tiles are cut from, and
		// stored in the DB; mosaic builds only parse the stored string. The mean commutes
		// with gamma-space downscaling, so the thumbnail gives the same value as the full
		// image at a fraction of the pixels.
		let mean = mean_displayed_srgb(&thumb).ok_or_else(|| anyhow::anyhow!("empty image"))?;
		let ucs = srgb_to_cam16ucs(mean, make_params());
		let cam16_str = format!(
			"{}{};{};{}",
			COLOR_STRING_PREFIX, ucs.lightness, ucs.a, ucs.b
		);

		debug!("Calculated color: {}", cam16_str);

		let mut full_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut full_png_bytes).write_image(
			img.as_raw(),
			img.width(),
			img.height(),
			ExtendedColorType::Rgba8,
		)?;

		let mut thumb_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut thumb_png_bytes).write_image(
			thumb.as_raw(),
			thumb.width(),
			thumb.height(),
			ExtendedColorType::Rgba8,
		)?;

		// Returns: color string, 128x128 thumbnail (for mosaic tiles), full-size PNG (for display)
		Ok((cam16_str, thumb_png_bytes, full_png_bytes))
	})
	.await
	.context("spawn_blocking panicked")?
}

#[cfg(test)]
mod tests {
	use super::*;

	fn image_of(pixels: &[(u8, u8, u8, u8)], width: u32) -> RgbaImage {
		let height = pixels.len() as u32 / width;
		let mut img = RgbaImage::new(width, height);
		for (i, (r, g, b, a)) in pixels.iter().enumerate() {
			img.put_pixel(
				i as u32 % width,
				i as u32 / width,
				image::Rgba([*r, *g, *b, *a]),
			);
		}
		img
	}

	#[test]
	fn solid_color_mean_equals_that_color() {
		let img = image_of(&vec![(192u8, 128, 64, 255); 64], 8);
		let mean = mean_displayed_srgb(&img).unwrap();

		assert!((mean.red - 192.0 / 255.0).abs() < 1e-4, "{mean:?}");
		assert!((mean.green - 128.0 / 255.0).abs() < 1e-4);
		assert!((mean.blue - 64.0 / 255.0).abs() < 1e-4);
	}

	#[test]
	fn high_contrast_tile_reads_as_its_displayed_gray() {
		// Half black, half white: a downscaler shows this as mid-gray (#808080). The
		// descriptor must agree with the displayed color, not with a perceptual average
		// (a mean of per-pixel CAM16-UCS lightness lands ~15 J' darker).
		let mut px = vec![(0u8, 0, 0, 255); 32];
		px.extend(vec![(255u8, 255, 255, 255); 32]);
		let mean = mean_displayed_srgb(&image_of(&px, 8)).unwrap();

		assert!((mean.red - 0.5).abs() < 0.01, "{mean:?}");
		assert!((mean.green - 0.5).abs() < 0.01);
		assert!((mean.blue - 0.5).abs() < 0.01);
	}

	#[test]
	fn transparent_pixels_read_as_white() {
		// A cutout avatar: 10 opaque dark pixels, 30 fully transparent ones (whatever their
		// hidden RGB is). Displayed on a white ground it reads mostly white, and the
		// descriptor must agree: (10*dark + 30*white) / 40.
		let mut px = vec![(10u8, 20, 30, 255); 10];
		px.extend(vec![(0u8, 0, 0, 0); 30]);
		let mean = mean_displayed_srgb(&image_of(&px, 8)).unwrap();

		let expected_r = (10.0 * 10.0 + 30.0 * 255.0) / 40.0 / 255.0;
		assert!((mean.red - expected_r).abs() < 1e-4, "{mean:?}");
		assert!(mean.green > 0.75 && mean.blue > 0.75, "{mean:?}");
	}

	#[test]
	fn fully_transparent_is_white() {
		let img = image_of(&vec![(255u8, 0, 0, 0); 16], 4);
		let mean = mean_displayed_srgb(&img).unwrap();
		assert!((mean.red - 1.0).abs() < 1e-4, "{mean:?}");
		assert!((mean.green - 1.0).abs() < 1e-4);
		assert!((mean.blue - 1.0).abs() < 1e-4);
	}

	#[test]
	fn composite_flattens_alpha_against_white() {
		let img = image_of(&[(0u8, 0, 0, 128), (100, 200, 50, 255), (9, 9, 9, 0)], 3);
		let flat = composite_over_white(&img);

		// 50%-alpha black blends to mid-gray; opaque stays; fully transparent becomes white
		assert_eq!(flat.get_pixel(0, 0).0, [127, 127, 127, 255]);
		assert_eq!(flat.get_pixel(1, 0).0, [100, 200, 50, 255]);
		assert_eq!(flat.get_pixel(2, 0).0, [255, 255, 255, 255]);
	}
}
