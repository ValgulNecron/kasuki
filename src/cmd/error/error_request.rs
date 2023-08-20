use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error::common::{edit_embed_message, get_localised_langage_edit};

pub async fn error_making_request_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    message: Message,
) {
    let localised_text =
        match get_localised_langage_edit(color, ctx, message.clone(), command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    edit_embed_message(
        color,
        ctx,
        message,
        localised_text.error_title.clone(),
        localised_text.error_request.clone(),
    )
    .await;
}
