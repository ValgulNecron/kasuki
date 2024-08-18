use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::command::server::generate_image_pfp_server;
use crate::config::{BotConfigDetails, Config};
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::structure::database::prelude::ServerImage;
use crate::structure::database::server_image::Column;
use crate::structure::message::server::generate_image_pfp_server::load_localization_pfp_server_image;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseMessage,
};
use serenity::builder::CreateInteractionResponseFollowup;
use tracing::trace;
use uuid::Uuid;
pub struct GenerateImagePfPCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for GenerateImagePfPCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for GenerateImagePfPCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        init(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn init(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    send_embed(
        ctx,
        command_interaction,
        "local",
        db_type,
        config.bot.config.clone(),
    )
    .await
}
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: &str,
    db_type: String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized text for the server's profile picture image
    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone(), db_config.clone()).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    // Retrieve the server's profile picture image
    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    let image = ServerImage::find()
        .filter(Column::ServerId.eq(guild_id.clone()))
        .filter(Column::ImageType.eq(image_type.to_string()))
        .one(&connection)
        .await?
        .ok_or(ResponseError::Option(format!(
            "Server image with type {} not found",
            image_type
        )))?
        .image;

    // Decode the image from base64
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64
        .decode(input)
        .map_err(|e| FollowupError::Decoding(format!("{:#?}", e)))?;

    // Generate a unique filename for the image
    let uuid = Uuid::new_v4();
    let image_path = format!("{}.png", uuid);

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    // Create an attachment with the image
    let attachment = CreateAttachment::bytes(image_data, image_path);

    // Construct the follow-up response with the embed and the attachment
    let builder = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    // Send the follow-up response to the command interaction
    command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
    trace!("Done");
    Ok(())
}
