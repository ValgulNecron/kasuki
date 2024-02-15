use crate::constant::COLOR;
use crate::database::sqlite::data::get_server_image_sqlite;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, Error};
use crate::error_enum::CommandError::{ErrorCommandSendingError, ErrorOptionError};
use crate::error_enum::DifferedCommandError::{DifferedCommandSendingError, WritingFile};
use crate::image_saver::general_image_saver::image_saver;
use crate::lang_struct::general::generate_image_pfp_server::load_localization_pfp_server_image;
use crate::server_image::calculate_user_color::{
    get_image_from_url, get_member, return_average_user_color,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Member, Timestamp,
};
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use tracing::{debug, error, trace};
use uuid::Uuid;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            Error(ErrorCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;
    let image = get_server_image_sqlite(&guild_id, &String::from("local"))
        .await?
        .1
        .unwrap_or_default();
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64.decode(input).unwrap();
    let uuid = Uuid::new_v4();

    let attachment = CreateAttachment::bytes(image_data, format!("{}.png", uuid.to_string()));

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!(
            "attachment://{}",
            format!("{}.png", uuid.to_string())
        ))
        .title(pfp_server_image_localised_text.title);

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            DifferedError(DifferedCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;
    trace!("Done");

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Color {
    pub cielab: Lab,
}

#[derive(Clone, Debug)]
pub struct ColorWithUrl {
    pub cielab: Lab,
    pub image: DynamicImage,
}

fn create_color_vector(tuples: Vec<(String, String, String)>) -> Vec<ColorWithUrl> {
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
