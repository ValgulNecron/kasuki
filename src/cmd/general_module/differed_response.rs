use std::fs;
use std::path::PathBuf;

use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;

pub async fn differed_response(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn differed_response_with_file_deletion(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    file_to_delete: PathBuf,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        let _ = fs::remove_file(&file_to_delete);
        println!("Cannot respond to slash command: {}", why);
    }
}
