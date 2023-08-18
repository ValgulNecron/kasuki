use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::lang_struct::TranslationLocalisedText;

pub async fn translation_embed(
    ctx: &Context,
    text: String,
    message: Message,
    localised_text: TranslationLocalisedText,
) {
    let color = Colour::FABLED_PINK;
    let mut real_message = message.clone();
    if let Err(why) = real_message
        .edit(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&localised_text.title)
                    .description(format!("{}", text))
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("{}: {}", &localised_text.error_slash_command, why);
    }
}
