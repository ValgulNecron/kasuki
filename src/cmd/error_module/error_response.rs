use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error_module::common::{edit_embed_message, get_localised_langage_edit};

pub async fn error_getting_response_from_url_edit(
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
        localised_text.error_getting_response_from_url.clone(),
    )
    .await;
}

pub async fn error_getting_bytes_response_edit(
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
        localised_text.error_getting_bytes.clone(),
    )
    .await;
}

pub async fn error_writing_file_response_edit(
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
        localised_text.error_writing_file.clone(),
    )
    .await;
}
