use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

use crate::function::error_management::common::{edit_embed_message, send_embed_message};

pub async fn no_langage_error(ctx: &Context, command: &ApplicationCommandInteraction) {
    println!("Error: Langage does not exist.");
    send_embed_message(ctx, command, "Error".to_string(), "Langage does not exist.").await;
}

pub async fn error_langage_file_not_found(ctx: &Context, command: &ApplicationCommandInteraction) {
    println!("Error: The langage file was not found.");
    send_embed_message(
        ctx,
        command,
        "Error".to_string(),
        "The langage file was not found.",
    )
    .await;
}

pub async fn error_cant_read_langage_file(ctx: &Context, command: &ApplicationCommandInteraction) {
    println!("Error: The langage file can't be read.");
    send_embed_message(
        ctx,
        command,
        "Error".to_string(),
        "The langage file can't be read.",
    )
    .await;
}

pub async fn error_parsing_langage_json(ctx: &Context, command: &ApplicationCommandInteraction) {
    println!("Error: Failed to parse the langage json file.");
    send_embed_message(
        ctx,
        command,
        "Error".to_string(),
        "Failed to parse the langage json file.",
    )
    .await;
}

pub async fn error_no_langage_guild_id(ctx: &Context, command: &ApplicationCommandInteraction) {
    println!("Error: Failed to get the guild id.");
    send_embed_message(
        ctx,
        command,
        "Error".to_string(),
        "Failed to get the guild id.",
    )
    .await;
}

pub async fn no_langage_error_edit(ctx: &Context, message: Message) {
    edit_embed_message(ctx, message, "Error".to_string(), "Langage does not exist.").await
}

pub async fn error_langage_file_not_found_edit(ctx: &Context, message: Message) {
    edit_embed_message(
        ctx,
        message,
        "Error".to_string(),
        "The langage file was not found.",
    )
    .await
}

pub async fn error_cant_read_langage_file_edit(ctx: &Context, message: Message) {
    edit_embed_message(
        ctx,
        message,
        "Error".to_string(),
        "The langage file can't be read.",
    )
    .await
}

pub async fn error_parsing_langage_json_edit(ctx: &Context, message: Message) {
    edit_embed_message(
        ctx,
        message,
        "Error".to_string(),
        "Failed to parse the json file.",
    )
    .await
}

pub async fn error_no_langage_guild_id_edit(ctx: &Context, message: Message) {
    edit_embed_message(
        ctx,
        message,
        "Error".to_string(),
        "Failed to get the guild id.",
    )
    .await
}
