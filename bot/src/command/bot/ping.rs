use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::bot::ping::load_localization_ping;
use crate::type_map_key::ShardManagerContainer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub struct PingCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for PingCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for PingCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized ping strings
    let ping_localised = load_localization_ping(guild_id, config.bot.config.clone()).await?;

    // Retrieve the shard manager from the context data
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Could not get the shard manager from the context data",
            ))));
        }
    }
    .runners
    .clone();

    // Lock the shard manager for exclusive access
    let shard_manager = shard_manager.lock().await;

    // Retrieve the shard ID from the context
    let shard_id = ctx.shard_id;

    // Retrieve the shard runner info from the shard manager
    let shard_runner_info = match shard_manager.get(&shard_id) {
        Some(data) => data,
        None => {
            return Err(Box::new(ResponseError::Option(String::from(
                "Could not get the shard runner info from the shard manager",
            ))));
        }
    };

    // Format the latency as a string
    let latency = match shard_runner_info.latency {
        Some(latency) => format!("{:.2}ms", latency.as_millis()),
        None => "?,??ms".to_string(),
    };

    // Retrieve the stage of the shard runner
    let stage = &shard_runner_info.stage.to_string();

    // Construct the embed for the response
    let builder_embed = get_default_embed(None)
        .description(
            ping_localised
                .desc
                .replace("$shard$", shard_id.to_string().as_str())
                .replace("$latency$", latency.as_str())
                .replace("$status$", stage),
        )
        .title(&ping_localised.title);

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
