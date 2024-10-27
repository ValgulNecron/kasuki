use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime::random_image::load_localization_random_image;
use anyhow::{anyhow, Result};
use image::EncodableLayout;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateAttachment,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use std::borrow::Cow;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeRandomImageCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for AnimeRandomImageCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for AnimeRandomImageCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        send(
            &self.ctx,
            &self.command_interaction,
            bot_data.config.clone(),
        )
        .await
    }
}

async fn send(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<()> {
    // Retrieve the type of image to fetch from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);

    let image_type = map
        .get(&String::from("image_type"))
        .ok_or(anyhow!("No image type specified"))?;

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized random image strings
    let random_image_localised =
        load_localization_random_image(guild_id, config.db.clone()).await?;

    // Create a deferred response to the command interaction
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    // Send the deferred response
    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;

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
pub async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    image_type: &String,
    title: String,
    endpoint: &str,
) -> Result<()> {
    // Construct the URL to fetch the image from
    let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);

    // Fetch the image from the URL
    let resp = reqwest::get(&url).await?;

    // Parse the response as JSON
    let json: serde_json::Value = resp.json().await?;

    // Retrieve the URL of the image from the JSON
    let image_url = json["url"]
        .as_str()
        .ok_or(anyhow!("No image found"))?
        .to_string();

    // Fetch the image from the image URL
    let response = reqwest::get(image_url).await?;

    // Retrieve the bytes of the image from the response
    let bytes = response.bytes().await?;

    // Generate a UUID for the filename of the image
    let uuid_name = Uuid::new_v4();

    let filename = format!("{}.gif", uuid_name);

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &filename))
        .title(title);

    // Construct the attachment for the image
    let bytes = bytes.as_bytes().to_vec();
    let cow_bytes = Cow::from(bytes);
    let attachment = CreateAttachment::bytes(cow_bytes, filename);

    // Construct the follow-up response containing the embed and the attachment
    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    // Send the follow-up response
    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await?;

    Ok(())
}
