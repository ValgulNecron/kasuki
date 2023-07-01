use serenity::builder::CreateApplicationCommand;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::struct_shard_manager::ShardManagerContainer;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
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


    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title("ping")
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(format!("Hey, I'm alive! \n You are running on the shard {}.\
                            \n Latency is {}", ctx.shard_id, latency))
                    })
                )
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}

