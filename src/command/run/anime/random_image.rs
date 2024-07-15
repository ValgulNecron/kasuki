use std::error::Error;
use std::fmt::format;
use std::sync::Arc;

use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use uuid::Uuid;

use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime::random_image::load_localization_random_image;

/// Executes the command to fetch and display a random image from the waifu.pics API.
///
/// This function retrieves the type of image to fetch from the command interaction and fetches a random image of that type from the waifu.pics API.
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
    // Load the localized random image strings
    let random_image_localised = load_localization_random_image(guild_id, db_type).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    // Send the random image as a response to the command interaction
    send_embed(
        ctx,
        command_interaction,
        image_type,
        random_image_localised.title,
        "sfw",
    )
    .await
}

/// Fetches a random image from the waifu.pics API and sends it as a response to a command interaction.
///
/// This function takes the type of image to fetch, the title for the response, and the endpoint to use on the waifu.pics API.
/// It fetches a random image of the specified type from the waifu.pics API and sends it as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `image_type` - The type of image to fetch.
/// * `title` - The title for the response.
/// * `endpoint` - The endpoint to use on the waifu.pics API.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: &String,
    title: String,
    endpoint: &str,
) -> Result<(), Box<dyn Error>> {
    // Construct the URL to fetch the image from
    let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);
    // Fetch the image from the URL
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| FollowupError::WebRequest(format!("{:#?}", e)))?;
    // Parse the response as JSON
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| FollowupError::Json(format!("{:#?}", e)))?;
    // Retrieve the URL of the image from the JSON
    let image_url = json["url"]
        .as_str()
        .ok_or("No image found")
        .map_err(|e| FollowupError::Json(format!("{:#?}", e)))?
        .to_string();

    // Fetch the image from the image URL
    let response = reqwest::get(image_url)
        .await
        .map_err(|e| FollowupError::WebRequest(format!("{:#?}", e)))?;
    // Retrieve the bytes of the image from the response
    let bytes = response
        .bytes()
        .await
        .map_err(|e| FollowupError::Byte(format!("{:#?}", e)))?;

    // Generate a UUID for the filename of the image
    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.gif", uuid_name);

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &filename))
        .title(title);

    // Construct the attachment for the image
    let attachment = CreateAttachment::bytes(bytes, &filename);

    // Construct the follow-up response containing the embed and the attachment
    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    // Send the follow-up response
    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;

    Ok(())
}
