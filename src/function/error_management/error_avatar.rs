use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::function::error_management::common::{get_localised_langage, send_embed_message};

pub async fn error_no_avatar(ctx: &Context, command: &ApplicationCommandInteraction) {
    let localised_text = match get_localised_langage(ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    println!("Error: Failed to fetch the discord user avatar.");
    send_embed_message(
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_no_avatar.as_str(),
    )
    .await;
}
