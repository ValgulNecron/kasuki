use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::function::error_management::common::{get_localised_langage, send_embed_message};

pub async fn error_file_type(ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_file_type.as_str(),
    )
    .await
}

pub async fn error_file_extension(ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_file_extension.as_str(),
    )
    .await
}
