use crate::function::error_management::common::{get_localised_langage, send_embed_message};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub async fn error_not_nsfw(ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_not_nsfw.as_str(),
    )
    .await
}
