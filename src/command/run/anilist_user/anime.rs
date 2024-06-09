use serenity::all::{CommandInteraction, Context};

use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::run::anilist::media::{send_embed, Media, MediaFormat, MediaType, MediaQuerryVariables, get_media};

/// This asynchronous function runs the command interaction for retrieving information about an anime.
///
/// It first retrieves the name or ID of the anime from the command interaction options.
///
/// If the value is an integer, it treats it as an ID and retrieves the anime with that ID.
/// If the value is not an integer, it treats it as a name and retrieves the anime with that name.
///
/// It sends an embed with the anime information as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the name or ID of the anime from the command interaction options
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    // If the value is an integer, treat it as an ID and retrieve the anime with that ID
    // If the value is not an integer, treat it as a name and retrieve the anime with that name
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = MediaQuerryVariables {
            format_in: Some(vec![
                Some(MediaFormat::Tv),
                Some(MediaFormat::TvShort),
                Some(MediaFormat::Movie),
                Some(MediaFormat::Special),
                Some(MediaFormat::Ova),
                Some(MediaFormat::Ona),
                Some(MediaFormat::Music),
            ]),
            id: Some(id),
            media_type: Some(MediaType::Anime),
            search: None,
        };
        get_media(id.to_string(), var).await?
    } else {
        let value_clone = value.clone();
        let var = MediaQuerryVariables {
            format_in: Some(vec![
                Some(MediaFormat::Tv),
                Some(MediaFormat::TvShort),
                Some(MediaFormat::Movie),
                Some(MediaFormat::Special),
                Some(MediaFormat::Ova),
                Some(MediaFormat::Ona),
                Some(MediaFormat::Music),
            ]),
            search: Some(&*value),
            media_type: Some(MediaType::Anime),
            id: None,
        };
        get_media(value_clone, var).await?
    };

    // Send an embed with the anime information as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}
