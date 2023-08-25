use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::cmd::error_module::common::{get_localised_langage, send_embed_message};

pub async fn error_no_avatar(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    println!("Error: Failed to fetch the discord user avatar.");
    send_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title.clone(),
        localised_text.error_no_avatar.clone(),
    )
    .await;
}
