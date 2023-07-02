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