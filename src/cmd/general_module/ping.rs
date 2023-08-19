
use serenity::builder::CreateApplicationCommand;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::struct_shard_manager::ShardManagerContainer;
use crate::cmd::lang_struct::embed::struct_lang_ping::PingLocalisedText;
use crate::cmd::lang_struct::register::struct_ping_register::RegisterLocalisedPing;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let latency = {
        let data_read = ctx.data.read().await;
        let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();

        let manager = shard_manager.lock().await;
        let runners = manager.runners.lock().await;

        let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();

        if let Some(duration) = runner.latency {
            format!("{:.2}ms", duration.as_millis())
        } else {
            "?ms".to_string()
        }
    };

    let color = Colour::FABLED_PINK;

    let localised_text = match PingLocalisedText::get_ping_localised(color, ctx, command).await
    {
        Ok(data) => data,
        Err(_) => return,
    };
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.title)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .color(color)
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
        {
            println!("Cannot respond to slash command: {}", why);
        }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let pings = RegisterLocalisedPing::get_ping_register_localised().unwrap();
    let command = command.name("ping").description("A ping command");
    for (_key, ping) in &pings {
        command
            .name_localized(&ping.code, &ping.ping)
            .description_localized(&ping.code, &ping.desc);
    }
    command
}
