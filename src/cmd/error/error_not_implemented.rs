use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error::common::{get_localised_langage, send_embed_message};

pub async fn error_not_implemented(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.not_implemented.clone(),
    )
    .await
}
