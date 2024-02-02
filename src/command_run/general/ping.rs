use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::COLOR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{ErrorCommandSendingError, ErrorOptionError};
use crate::lang_struct::general::ping::load_localization_ping;
use crate::struct_shard_manager::ShardManagerContainer;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let ping_localised = load_localization_ping(guild_id).await?;
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => return Err(Error(ErrorOptionError(String::from("There is no option")))),
    }
    .runners
    .clone();
    let shard_manager = shard_manager.lock().await;

    let shard_id = ctx.shard_id;

    let shard_runner_info = match shard_manager.get(&shard_id) {
        Some(data) => data,
        None => return Err(Error(ErrorOptionError(String::from("There is no option")))),
    };

    let latency = match shard_runner_info.latency {
        Some(latency) => format!("{:.2}ms", latency.as_millis()),
        None => "?,??ms".to_string(),
    };

    let tx_status = &shard_runner_info.stage.to_string();

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(
            ping_localised
                .desc
                .replace("$shard$", shard_id.to_string().as_str())
                .replace("$latency$", latency.as_str())
                .replace("$status$", tx_status),
        )
        .title(&ping_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            Error(ErrorCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })
}
