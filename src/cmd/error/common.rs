use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::error::no_lang_error::{
    error_cant_read_langage_file, error_cant_read_langage_file_edit, error_langage_file_not_found,
    error_langage_file_not_found_edit, error_no_langage_guild_id, error_no_langage_guild_id_edit,
    error_parsing_langage_json, error_parsing_langage_json_edit, no_langage_error,
    no_langage_error_edit,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::lang_struct::embed::error::ErrorLocalisedText;

pub async fn send_embed_message(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    title: String,
    desc: String,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(title)
                            .description(desc)
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn edit_embed_message(
    color: Colour,
    ctx: &Context,
    mut message: Message,
    title: String,
    desc: String,
) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title(title)
                    .description(desc)
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn send_followup_embed_message(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    title: String,
    desc: String,
) {
    if let Err(why) = command
        .create_followup_message(&ctx.http, |message| {
            message.embed(|m| {
                m.title(title)
                    .description(desc)
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn get_localised_langage(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<ErrorLocalisedText, &'static str> {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_langage_file_not_found(color, ctx, command).await;
            return Err("not found");
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => {
            error_cant_read_langage_file(color, ctx, command).await;
            return Err("not found");
        }
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_langage_json(color, ctx, command).await;
            return Err("not found");
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_langage_guild_id(color, ctx, command).await;
            return Err("not found");
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    return if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        Ok(localised_text.clone())
    } else {
        no_langage_error(color, ctx, command).await;
        Err("not found")
    };
}

pub async fn get_localised_langage_edit(
    color: Colour,
    ctx: &Context,
    message: Message,
    command: &ApplicationCommandInteraction,
) -> Result<ErrorLocalisedText, &'static str> {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_langage_file_not_found_edit(color, ctx, message.clone()).await;
            return Err("not found");
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => {
            error_cant_read_langage_file_edit(color, ctx, message.clone()).await;
            return Err("not found");
        }
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_langage_json_edit(color, ctx, message.clone()).await;
            return Err("not found");
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_langage_guild_id_edit(color, ctx, message.clone()).await;
            return Err("not found");
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    return if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        Ok(localised_text.clone())
    } else {
        no_langage_error_edit(color, ctx, message.clone()).await;
        Err("not found")
    };
}

pub async fn custom_error(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error: &String,
) {
    let localised_text = match get_localised_langage(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    send_embed_message(
        color,
        ctx,
        command,
        localised_text.error_title,
        error.clone(),
    )
    .await;
}
