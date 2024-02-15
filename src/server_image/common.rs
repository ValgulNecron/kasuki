use crate::database_struct::user_color_struct::UserColor;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use image::DynamicImage;
use palette::{IntoColor, Lab, Srgb};
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct Color {
    pub cielab: Lab,
}

#[derive(Clone, Debug)]
pub struct ColorWithUrl {
    pub cielab: Lab,
    pub image: DynamicImage,
}

pub fn create_color_vector_from_tuple(tuples: Vec<(String, String, String)>) -> Vec<ColorWithUrl> {
    tuples
        .into_iter()
        .filter_map(|(hex, _, image)| {
            let r = hex[1..3].parse::<u8>();
            let g = hex[3..5].parse::<u8>();
            let b = hex[5..7].parse::<u8>();

            let input = image.trim_start_matches("data:image/png;base64,");
            let decoded = BASE64.decode(input).unwrap();
            let img = image::load_from_memory(&decoded).unwrap();

            get_color_with_url(img, r, g, b)
        })
        .collect()
}

pub fn create_color_vector_from_user_color(tuples: Vec<UserColor>) -> Vec<ColorWithUrl> {
    tuples
        .into_iter()
        .filter_map(|user_color| {
            let hex = user_color.color;
            let image = user_color.image;
            let hex = hex.unwrap_or_default();
            let r = hex[1..3].parse::<u8>();
            let g = hex[3..5].parse::<u8>();
            let b = hex[5..7].parse::<u8>();

            let image = image.unwrap_or_default();
            let input = image.trim_start_matches("data:image/png;base64,");
            let decoded = BASE64.decode(input).unwrap();
            let img = image::load_from_memory(&decoded).unwrap();

            get_color_with_url(img, r, g, b)
        })
        .collect()
}

pub fn find_closest_color(colors: &[ColorWithUrl], target: &Color) -> Option<ColorWithUrl> {
    let a = colors.iter().min_by(|&a, &b| {
        let delta_l = (a.cielab.l - target.cielab.l).abs();
        let delta_a = (a.cielab.a - target.cielab.a).abs();
        let delta_b = (a.cielab.b - target.cielab.b).abs();
        let delta_e_a = (delta_l.powi(2) + delta_a.powi(2) + delta_b.powi(2)).sqrt();

        let delta_l = (b.cielab.l - target.cielab.l).abs();
        let delta_a = (b.cielab.a - target.cielab.a).abs();
        let delta_b = (b.cielab.b - target.cielab.b).abs();
        let delta_e_b = (delta_l.powi(2) + delta_a.powi(2) + delta_b.powi(2)).sqrt();

        delta_e_a.partial_cmp(&delta_e_b).unwrap()
    });
    a.cloned()
}

pub fn get_color_with_url(
    img: DynamicImage,
    r: Result<u8, ParseIntError>,
    g: Result<u8, ParseIntError>,
    b: Result<u8, ParseIntError>,
) -> Option<ColorWithUrl> {
    match (r, g, b) {
        (Ok(r), Ok(g), Ok(b)) => {
            let r_normalized = r as f32 / 255.0;
            let g_normalized = g as f32 / 255.0;
            let b_normalized = b as f32 / 255.0;
            let rgb_color = Srgb::new(r_normalized, g_normalized, b_normalized);
            let lab_color: Lab = rgb_color.into_color();
            Some(ColorWithUrl {
                cielab: lab_color,
                image: img,
            })
        }
        _ => None,
    }
}
