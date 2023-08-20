use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::lang_struct::embed::general::struct_lang_in_progress::InProgressLocalisedText;

pub async fn in_progress_embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<Option<Message>, String> {
    let color = Colour::FABLED_PINK;
    let localised_text =
        match InProgressLocalisedText::get_in_progress_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(data) => return Err(data.parse().unwrap()),
        };
    let message = command
        .create_followup_message(&ctx.http, |f| {
            f.embed(|e| {
                e.title(&localised_text.title)
                    .description(&localised_text.description)
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await;
    Ok(Some(message.unwrap()))
}
