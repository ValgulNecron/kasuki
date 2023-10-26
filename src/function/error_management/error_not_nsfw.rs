use crate::function::error_management::common::{get_localised_langage, send_embed_message};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

pub async fn error_not_nsfw(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.forgot_module.clone(),
    )
    .await
}
