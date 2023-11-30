use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::structure::general::ping::load_localization_ping;
use crate::structure::struct_shard_manager::ShardManagerContainer;

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();
    let ping_localised = load_localization_ping(guild_id).await?;
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => return Err(OPTION_ERROR.clone()),
    }
    .runners
    .clone();
    let shard_manager = shard_manager.lock().await;

    let shard_id = ctx.shard_id;

    let shard_runner_info = match shard_manager.get(&shard_id) {
        Some(data) => data,
        None => return Err(OPTION_ERROR.clone()),
    };

    let latency = match shard_runner_info.latency {
        Some(latency) => format!("{:.2}ms", latency.as_millis()),
        None => format!("?,??ms"),
    };

    let tx_status = &shard_runner_info.stage.to_string();

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(
            &ping_localised
                .desc
                .replace("$shard$", shard_id.to_string().as_str())
                .replace("$latency$", latency.as_str())
                .replace("$status$", tx_status),
        )
        .title(&ping_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
