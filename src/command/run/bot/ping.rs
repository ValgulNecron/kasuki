use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::struct_shard_manager::ShardManagerContainer;
use crate::structure::message::bot::ping::load_localization_ping;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;

/// Executes the command to display the bot's ping.
///
/// This function retrieves the localized ping strings and formats them into a response to the command interaction.
/// The response includes the bot's shard ID, latency, and status, which are sent as an embed.
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
) -> Result<(), AppError> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized ping strings
    let ping_localised = load_localization_ping(guild_id).await?;

    // Retrieve the shard manager from the context data
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return Err(AppError::new(
                String::from("Could not get the shard manager from the data"),
                ErrorType::Option,
                ErrorResponseType::Message,
            ));
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
            return Err(AppError::new(
                String::from("Could not get the shard info from the shard manager"),
                ErrorType::Option,
                ErrorResponseType::Message,
            ));
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
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
