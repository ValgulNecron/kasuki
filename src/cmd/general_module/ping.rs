use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::builder::CreateApplicationCommand;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::PingLocalisedText;
use crate::cmd::general_module::struct_shard_manager::ShardManagerContainer;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
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

    let mut file = File::open("lang_file/general/ping_lang.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, PingLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(&localised_text.title)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .color(color)
                                .description(format!("{}{}{}{}{}", &localised_text.description_part_1,
                                                     &localised_text.description_part_2 ,ctx.shard_id,
                                                     &localised_text.description_part_3, latency))
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}

