use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::function::error_management::common::{edit_embed_message, get_localised_langage_edit};

pub async fn error_making_request_edit(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    message: Message,
) {
    let localised_text = match get_localised_langage_edit(ctx, message.clone(), command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    edit_embed_message(
        ctx,
        message,
        localised_text.error_title.clone(),
        localised_text.error_request.as_str(),
    )
    .await;
}
