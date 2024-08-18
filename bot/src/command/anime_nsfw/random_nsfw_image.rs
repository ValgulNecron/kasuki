use std::error::Error;
use std::sync::Arc;

use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponseMessage};

use crate::command::anime::random_image::send_embed;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anime_nsfw::random_image_nsfw::load_localization_random_image_nsfw;

pub struct AnimeRandomNsfwImageCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for AnimeRandomNsfwImageCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for AnimeRandomNsfwImageCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}
async fn send(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
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
        load_localization_random_image_nsfw(guild_id, config.bot.config.clone()).await?;

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
