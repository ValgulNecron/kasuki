use crate::function::error_management::common::{edit_embed_message, get_localised_langage_edit};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;

pub async fn error_parsing_json_edit(
    ctx: &Context,
    message: Message,
    command: &ApplicationCommandInteraction,
) {
    let localised_text = match get_localised_langage_edit(ctx, message.clone(), command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    edit_embed_message(
        ctx,
        message,
        localised_text.error_title,
        localised_text.error_parsing_json.as_str(),
    )
    .await
}
