use std::num::ParseIntError;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use image::DynamicImage;
use palette::color_difference::ImprovedDeltaE;
use palette::{IntoColor, Lab, Srgb};

use crate::database::data_struct::user_color::UserColor;

/// `Color` is a struct that represents a color in the CIELAB color space.
/// It contains a single field, `cielab`, which is a `Lab` object from the `palette` crate.
#[derive(Clone, Debug)]
pub struct Color {
    pub cielab: Lab,
}

/// `ColorWithUrl` is a struct that represents a color in the CIELAB color space,
/// along with an associated image. It contains two fields: `cielab`, which is a `Lab`
/// object from the `palette` crate, and `image`, which is a `DynamicImage` object from
/// the `image` crate.
#[derive(Clone, Debug)]
pub struct ColorWithUrl {
    pub cielab: Lab,
    pub image: DynamicImage,
}

/// `create_color_vector_from_tuple` is a function that takes a vector of tuples,
/// where each tuple contains three strings representing a hex color code and an image
/// in base64 format. It returns a vector of `ColorWithUrl` objects, where each object
/// represents a color and an associated image.
///
/// # Arguments
///
/// * `tuples` - A vector of tuples, where each tuple contains three strings. The first
/// string is a hex color code, and the third string is an image in base64 format.
pub fn create_color_vector_from_tuple(tuples: Vec<(String, String, String)>) -> Vec<ColorWithUrl> {
    tuples
        .into_iter()
        .filter_map(|(hex, _, image)| {
            let (r, g, b) = (
                hex[1..3].parse::<u8>(),
                hex[3..5].parse::<u8>(),
                hex[5..7].parse::<u8>(),
            );
            let input = image.trim_start_matches("data:image/png;base64,");
            let img = image::load_from_memory(&BASE64.decode(input).unwrap()).unwrap();
            get_color_with_url(img, r, g, b)
        })
        .collect()
}

/// `create_color_vector_from_user_color` is a function that takes a vector of `UserColor`
/// objects and returns a vector of `ColorWithUrl` objects, where each object represents
/// a color and an associated image.
///
/// # Arguments
///
/// * `tuples` - A vector of `UserColor` objects.
pub fn create_color_vector_from_user_color(tuples: Vec<UserColor>) -> Vec<ColorWithUrl> {
    tuples
        .into_iter()
        .filter_map(|user_color| {
            let hex = user_color.color.unwrap_or_default();
            let image = user_color.image.unwrap_or_default();

            let (r, g, b) = (
                hex[1..3].parse::<u8>(),
                hex[3..5].parse::<u8>(),
                hex[5..7].parse::<u8>(),
            );

            let input = image.trim_start_matches("data:image/png;base64,");
            let decoded = BASE64.decode(input).unwrap();
            let img = image::load_from_memory(&decoded).unwrap();

            get_color_with_url(img, r, g, b)
        })
        .collect()
}

/// This function finds the color in the provided list that is closest to the target color.
///
/// # Arguments
///
/// * `colors` - A slice of `ColorWithUrl` objects.
/// * `target` - A reference to a `Color` object that we want to find the closest match for.
///
/// # Returns
///
/// * `Option<ColorWithUrl>` - Returns an optional `ColorWithUrl`. If a closest color is found, it returns `Some(ColorWithUrl)`. If the colors list is empty, it returns `None`.
pub fn find_closest_color(colors: &[ColorWithUrl], target: &Color) -> Option<ColorWithUrl> {
    colors
        .iter()
        .min_by(|&a, &b| {
            let delta_e_a = a.cielab.improved_delta_e(target.cielab);
            let delta_e_b = b.cielab.improved_delta_e(target.cielab);
            delta_e_a.partial_cmp(&delta_e_b).unwrap()
        })
        .cloned()
}

/// This function creates a `ColorWithUrl` object from an image and RGB color values.
///
/// # Arguments
///
/// * `img` - A `DynamicImage` object.
/// * `r` - A `Result` object that contains either a `u8` value for the red color component or a `ParseIntError`.
/// * `g` - A `Result` object that contains either a `u8` value for the green color component or a `ParseIntError`.
/// * `b` - A `Result` object that contains either a `u8` value for the blue color component or a `ParseIntError`.
///
/// # Returns
///
/// * `Option<ColorWithUrl>` - Returns an optional `ColorWithUrl`. If the RGB color values are valid, it returns `Some(ColorWithUrl)`. If any of the RGB color values are invalid, it returns `None`.
pub fn get_color_with_url(
    img: DynamicImage,
    r: Result<u8, ParseIntError>,
    g: Result<u8, ParseIntError>,
    b: Result<u8, ParseIntError>,
) -> Option<ColorWithUrl> {
    if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
        let rgb_color = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let lab_color: Lab = rgb_color.into_color();
        Some(ColorWithUrl {
            cielab: lab_color,
            image: img,
        })
    } else {
        None
    }
}
