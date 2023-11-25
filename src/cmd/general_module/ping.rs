use crate::constant::COLOR;
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::structure::embed::general::struct_lang_ping::PingLocalisedText;
use crate::structure::register::general::struct_ping_register::RegisterLocalisedPing;
use serenity::builder::CreateApplicationCommand;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;
use crate::error_enum::AppError::LangageGuildIdError;

use crate::structure::struct_shard_manager::ShardManagerContainer;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), AppError> {
    let data_read = ctx.data.read().await;
    let shard_manager = data_read
        .get::<ShardManagerContainer>()
        .ok_or(OPTION_ERROR.clone())?;

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = runners.get(&ShardId(ctx.shard_id)).ok_or(OPTION_ERROR.clone())?;

    let latency = match runner.latency {
        Some(duration) => format!("{:.2}ms", duration.as_millis()),
        None => "?ms".to_string(),
    };

    let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                return Err(LangageGuildIdError(String::from("Guild id for langage not found.")));
            }
        };

    let localised_text = PingLocalisedText::get_ping_localised(guild_id).await?;
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(&localised_text.title)
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .description(format!(
                                "{}{}{}{}{}",
                                &localised_text.description_part_1,
                                &localised_text.description_part_2,
                                ctx.shard_id,
                                &localised_text.description_part_3,
                                latency
                            ))
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let pings = RegisterLocalisedPing::get_ping_register_localised().unwrap();
    command.name("ping").description("A ping command");
    for ping in pings.values() {
        command
            .name_localized(&ping.code, &ping.name)
            .description_localized(&ping.code, &ping.desc);
    }
    command
}
