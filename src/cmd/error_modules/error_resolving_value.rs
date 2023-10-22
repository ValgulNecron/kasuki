use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error_modules::common::{get_localised_langage, send_followup_embed_message};

pub async fn error_resolving_value_followup(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_followup_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_resolving_value.clone(),
    )
    .await
}
