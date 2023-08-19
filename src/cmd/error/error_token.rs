use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error::common::{edit_embed_message, get_localised_langage, get_localised_langage_edit, send_embed_message};

pub async fn error_no_token(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.no_token.clone(),
    ).await
}

pub async fn error_no_token_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    message: Message,
) {
    let localised_text = match get_localised_langage_edit(color, ctx, message.clone(), command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    edit_embed_message(color, ctx, message, localised_text.error_title.clone(), localised_text.no_token.clone()).await;
}