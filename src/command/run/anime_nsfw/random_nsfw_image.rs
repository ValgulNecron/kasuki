use std::error::Error;
use std::sync::Arc;

use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponseMessage};

use crate::command::run::anime::random_image::send_embed;
use crate::config::Config;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime_nsfw::random_image_nsfw::load_localization_random_image_nsfw;

/// Executes the command to fetch and display a random NSFW image from the waifu.pics API.
///
/// This function retrieves the type of image to fetch from the command interaction and fetches a random NSFW image of that type from the waifu.pics API.
/// It then sends the image as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the type of image to fetch from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let image_type = map
        .get(&String::from("image_type"))
        .ok_or(ResponseError::Option(String::from(
            "No image type specified",
        )))?;

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized random NSFW image strings
    let random_image_nsfw_localised =
        load_localization_random_image_nsfw(guild_id, db_type).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    // Send the random NSFW image as a response to the command interaction
    send_embed(
        ctx,
        command_interaction,
        image_type,
        random_image_nsfw_localised.title,
        "nsfw",
    )
    .await
}
