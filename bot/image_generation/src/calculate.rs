use anyhow::{Context, Result};
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder, ImageReader};
use palette::{
	Clamp, FromColor, IntoColor, Lab, LinSrgb, LinSrgba, Srgb, Srgba, Xyz,
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

const CAT16: [[f32; 3]; 3] = [
	[0.401288, 0.650173, -0.051461],
	[-0.250268, 1.204414, 0.045854],
	[-0.002079, 0.048952, 0.953127],
];

const CAT16_INV: [[f32; 3]; 3] = [
	[1.862068, -1.011255, 0.149187],
	[0.387526, 0.621447, -0.008973],
	[-0.015885, -0.034197, 1.049082],
];

fn mat3_mul_vec3(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
	[
		m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
		m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
		m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
	]
}

const D65_XYZ: [f32; 3] = [0.95047, 1.00000, 1.08883];

fn estimate_white_point(linear_pixels: &[LinSrgba<f32>]) -> [f32; 3] {
	// Exclude fully transparent pixels
	let xyz_pixels: Vec<[f32; 3]> = linear_pixels
		.iter()
		.filter(|p| p.alpha > 0.01)
		.map(|p| {
			let xyz: Xyz<D65, f32> = (*p).into_color();
			[xyz.x, xyz.y, xyz.z]
		})
		.collect();

	if xyz_pixels.is_empty() {
		return D65_XYZ; // fallback: assume already D65
	}

	// Sort by luminance (Y) descending
	let mut indexed: Vec<(usize, f32)> = xyz_pixels
		.iter()
		.enumerate()
		.map(|(i, xyz)| (i, xyz[1]))
		.collect();
	indexed.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

	let top_n = (indexed.len() / 100).max(1);
	let (sum_x, sum_y, sum_z) =
		indexed[..top_n]
			.iter()
			.fold((0.0f32, 0.0f32, 0.0f32), |acc, (i, _)| {
				(
					acc.0 + xyz_pixels[*i][0],
					acc.1 + xyz_pixels[*i][1],
					acc.2 + xyz_pixels[*i][2],
				)
			});

	let n = top_n as f32;
	let y = sum_y / n;

	if y < 1e-6 {
		return D65_XYZ; // image is black, avoid div-by-zero
	}

	[sum_x / n / y, 1.0, sum_z / n / y]
}

fn apply_cat16_to_pixels_par(linear_pixels: &mut [LinSrgba<f32>]) {
	// &mut [T] not Vec
	let src_wp = estimate_white_point(linear_pixels);

	// If the estimated white is already D65, skip the whole pass
	let diff = (src_wp[0] - D65_XYZ[0]).abs() + (src_wp[2] - D65_XYZ[2]).abs(); // Y is always 1.0
	if diff < 1e-4 {
		return;
	}

	let lms_src = mat3_mul_vec3(&CAT16, src_wp);
	let lms_dst = mat3_mul_vec3(&CAT16, D65_XYZ);
	let scale = [
		lms_dst[0] / lms_src[0],
		lms_dst[1] / lms_src[1],
		lms_dst[2] / lms_src[2],
	];

	linear_pixels.par_iter_mut().for_each(|pixel| {
		let a = pixel.alpha; // preserve alpha, don't adapt it
		let xyz: Xyz<D65, f32> = (*pixel).into_color();
		let lms = mat3_mul_vec3(&CAT16, [xyz.x, xyz.y, xyz.z]);
		let lms_a = [lms[0] * scale[0], lms[1] * scale[1], lms[2] * scale[2]];
		let [x, y, z] = mat3_mul_vec3(&CAT16_INV, lms_a);
		let mut adapted: LinSrgba<f32> = Xyz::<D65, f32>::new(x, y, z).into_color();
		adapted = Clamp::clamp(adapted);
		adapted.alpha = a;
		*pixel = adapted;
	});
}

pub fn make_params() -> BakedParameters<StaticWp<D65>, f32> {
	let mut p = Parameters::default_static_wp(64.0f32); // adapting_luminance
	p.background_luminance = 0.2;
	p.surround = Surround::Average;
	p.bake()
}

fn to_cam16ucs_par(linear_pixels: &[LinSrgba<f32>]) -> Vec<Option<Cam16UcsJab<f32>>> {
	let params = make_params();

	linear_pixels
		.par_iter()
		.map(|pixel| {
			if pixel.alpha < 0.1 {
				return None;
			}

			let rgb = LinSrgb::new(
				pixel.red * pixel.alpha,
				pixel.green * pixel.alpha,
				pixel.blue * pixel.alpha,
			);

			let xyz: Xyz<D65, f32> = rgb.into_color();
			let cam16 = Cam16::from_xyz(xyz, params);
			Some(Cam16UcsJab::from_color(cam16))
		})
		.collect()
}

fn is_chromatic(px: &Cam16UcsJab<f32>) -> bool {
	let chroma = (px.a * px.a + px.b * px.b).sqrt();
	let j = px.lightness;
	chroma > 8.0 && j > 10.0 && j < 95.0
}

fn center_saliency(row: usize, col: usize, height: usize, width: usize) -> f32 {
	let dy = (row as f32 - height as f32 * 0.5) / (height as f32 * 0.5);
	let dx = (col as f32 - width as f32 * 0.5) / (width as f32 * 0.5);
	let sigma2 = 0.5f32 * 0.5;
	(-(dx * dx + dy * dy) / (2.0 * sigma2)).exp()
}

#[derive(Clone, Copy, Debug)]
struct Jab {
	j: f32,
	a: f32,
	b: f32,
}

impl Jab {
	fn dist_sq(&self, other: &Jab) -> f32 {
		let dj = self.j - other.j;
		let da = self.a - other.a;
		let db = self.b - other.b;
		dj * dj + da * da + db * db
	}
}

fn weighted_kmeans(points: &[(Jab, f32)], k: usize, max_iters: usize) -> Vec<Jab> {
	if points.is_empty() || k == 0 {
		return vec![];
	}
	let k = k.min(points.len());

	let mut centroids: Vec<Jab> = Vec::with_capacity(k);

	let first = points
		.iter()
		.max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
		.unwrap()
		.0;
	centroids.push(first);

	for _ in 1..k {
		let probs: Vec<f32> = points
			.iter()
			.map(|(jab, w)| {
				let min_d2 = centroids
					.iter()
					.map(|c| jab.dist_sq(c))
					.fold(f32::INFINITY, f32::min);
				w * min_d2
			})
			.collect();

		let total: f32 = probs.iter().sum();
		if total < 1e-10 {
			break;
		}

		let next = points
			.iter()
			.zip(probs.iter())
			.max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
			.unwrap()
			.0
			.0;
		centroids.push(next);
	}

	let mut assignments = vec![0usize; points.len()];
	for _ in 0..max_iters {
		let new_assignments: Vec<usize> = points
			.par_iter()
			.map(|(jab, _)| {
				centroids
					.iter()
					.enumerate()
					.min_by(|(_, ca), (_, cb)| {
						jab.dist_sq(ca).partial_cmp(&jab.dist_sq(cb)).unwrap()
					})
					.unwrap()
					.0
			})
			.collect();

		let changed = new_assignments != assignments;
		assignments = new_assignments;

		let mut sum_j = vec![0.0f32; k];
		let mut sum_a = vec![0.0f32; k];
		let mut sum_b = vec![0.0f32; k];
		let mut sum_w = vec![0.0f32; k];

		for (idx, (jab, w)) in assignments.iter().zip(points.iter()) {
			sum_j[*idx] += jab.j * w;
			sum_a[*idx] += jab.a * w;
			sum_b[*idx] += jab.b * w;
			sum_w[*idx] += w;
		}

		for (i, c) in centroids.iter_mut().enumerate() {
			if sum_w[i] > 1e-10 {
				c.j = sum_j[i] / sum_w[i];
				c.a = sum_a[i] / sum_w[i];
				c.b = sum_b[i] / sum_w[i];
			}
		}

		if !changed {
			break;
		}
	}

	let mut cluster_weights = vec![0.0f32; k];
	for (idx, (_, w)) in assignments.iter().zip(points.iter()) {
		cluster_weights[*idx] += w;
	}
	let mut indexed: Vec<(usize, f32)> = cluster_weights.into_iter().enumerate().collect();
	indexed.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

	indexed.iter().map(|(i, _)| centroids[*i]).collect()
}

pub fn dominant_cam16_color(
	cam16_pixels: &[Option<Cam16UcsJab<f32>>], img_width: u32, img_height: u32, k: usize,
) -> Option<Cam16UcsJab<f32>> {
	let width = img_width as usize;
	let height = img_height as usize;

	let points: Vec<(Jab, f32)> = cam16_pixels
		.par_iter()
		.enumerate()
		.filter_map(|(i, px)| {
			let px = px.as_ref()?;
			if !is_chromatic(px) {
				return None;
			}
			let row = i / width;
			let col = i % width;
			let saliency = center_saliency(row, col, height, width);
			Some((
				Jab {
					j: px.lightness,
					a: px.a,
					b: px.b,
				},
				saliency,
			))
		})
		.collect();
	if points.is_empty() {
		return None;
	}

	let centroids = weighted_kmeans(&points, k, 50);
	centroids.first().map(|c| Cam16UcsJab {
		lightness: c.j,
		a: c.a,
		b: c.b,
	})
}

// CAM16-UCS is only used to *pick* colors; every downstream consumer (delta E matching,
// hex rendering) works from XYZ, so funnel both through one inverse transform.
pub fn cam16ucs_to_xyz(color: Cam16UcsJab<f32>) -> Xyz<D65, f32> {
	let params = make_params();
	let jmh = Cam16Jmh::from_color(color);
	let cam16: Cam16<f32> = jmh.into_full(params);
	cam16.into_xyz(params)
}

// Color *matching* is done with CIEDE2000 in CIELAB, so the CAM16-derived mean color has to
// come back out to Lab before it can be compared against anything.
pub fn cam16ucs_to_lab(color: Cam16UcsJab<f32>) -> Lab<D65, f32> {
	Lab::from_color(cam16ucs_to_xyz(color))
}

pub fn cam16ucs_to_srgb_u8(dominant: Cam16UcsJab<f32>) -> [u8; 3] {
	let xyz = cam16ucs_to_xyz(dominant);
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
		let (img_width, img_height) = img.dimensions();
		let img = img.to_rgba8();
		let raw: &[u8] = img.as_raw();
		let srgba_pixels: &[Srgba<u8>] = raw.components_as();
		let mut linear_pixels: Vec<LinSrgba<f32>> =
			srgba_pixels.par_iter().map(|p| p.into_linear()).collect();

		apply_cat16_to_pixels_par(&mut linear_pixels);

		let cam16_pixels: Vec<Option<Cam16UcsJab<f32>>> = to_cam16ucs_par(&linear_pixels);

		let dominant = dominant_cam16_color(&cam16_pixels, img_width, img_height, 1)
			.ok_or_else(|| anyhow::anyhow!("image is fully achromatic"))?;
		//let [r, g, b] = cam16ucs_to_srgb_u8(dominant);
		//let hex = cam16ucs_to_hex(r, g, b);
		let cam16_str = format!("cam16;{};{};{}", dominant.lightness, dominant.a, dominant.b);

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
