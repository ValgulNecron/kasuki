use crate::function::error_management::common::{edit_embed_message, send_embed_message};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

pub async fn no_langage_error(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    println!("Error: Langage does not exist.");
    send_embed_message(
        color,
        ctx,
        command,
        "Error".to_string(),
        "Langage does not exist.".to_string(),
    )
    .await;
}

pub async fn error_langage_file_not_found(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    println!("Error: The langage file was not found.");
    send_embed_message(
        color,
        ctx,
        command,
        "Error".to_string(),
        "The langage file was not found.".to_string(),
    )
    .await;
}

pub async fn error_cant_read_langage_file(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    println!("Error: The langage file can't be read.");
    send_embed_message(
        color,
        ctx,
        command,
        "Error".to_string(),
        "The langage file can't be read.".to_string(),
    )
    .await;
}

pub async fn error_parsing_langage_json(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    println!("Error: Failed to parse the langage json file.");
    send_embed_message(
        color,
        ctx,
        command,
        "Error".to_string(),
        "Failed to parse the langage json file.".to_string(),
    )
    .await;
}

pub async fn error_no_langage_guild_id(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    println!("Error: Failed to get the guild id.");
    send_embed_message(
        color,
        ctx,
        command,
        "Error".to_string(),
        "Failed to get the guild id.".to_string(),
    )
    .await;
}

pub async fn no_langage_error_edit(color: Colour, ctx: &Context, message: Message) {
    edit_embed_message(
        color,
        ctx,
        message,
        "Error".to_string(),
        "Langage does not exist.".to_string(),
    )
    .await
}

pub async fn error_langage_file_not_found_edit(color: Colour, ctx: &Context, message: Message) {
    edit_embed_message(
        color,
        ctx,
        message,
        "Error".to_string(),
        "The langage file was not found.".to_string(),
    )
    .await
}

pub async fn error_cant_read_langage_file_edit(color: Colour, ctx: &Context, message: Message) {
    edit_embed_message(
        color,
        ctx,
        message,
        "Error".to_string(),
        "The langage file can't be read.".to_string(),
    )
    .await
}

pub async fn error_parsing_langage_json_edit(color: Colour, ctx: &Context, message: Message) {
    edit_embed_message(
        color,
        ctx,
        message,
        "Error".to_string(),
        "Failed to parse the json file.".to_string(),
    )
    .await
}

pub async fn error_no_langage_guild_id_edit(color: Colour, ctx: &Context, message: Message) {
    edit_embed_message(
        color,
        ctx,
        message,
        "Error".to_string(),
        "Failed to get the guild id.".to_string(),
    )
    .await
}
