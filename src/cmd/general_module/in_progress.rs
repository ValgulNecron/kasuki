use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn in_progress_embed(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<Message> {
    let color = Colour::FABLED_PINK;

    let message = command
        .create_followup_message(&ctx.http, |f| {
            f.embed(|e| e.title("In progress")
                .description("The task is being processed...be patient, it may take some time!")
                .timestamp(Timestamp::now())
                .color(color))
        })
        .await;

    return message;
}