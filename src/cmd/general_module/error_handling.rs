use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;
use serenity::Error;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::ErrorLocalisedText;

pub async fn error_message(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &String,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}", error_message))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_followup_message(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &String,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_followup_message(&ctx.http, |message| {
                message.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(format!("{}", error_message))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn no_langage_error(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("Langage does not exist.")
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

pub async fn error_message_with_why(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &String,
    why: Error,
) {
    let mut file = File::open("lang_file/embed/error.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, ErrorLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id.clone()).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}: {}", error_message, why))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_file_not_found(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("The langage file was not found.")
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

pub async fn error_cant_read_file(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("The langage file can't be read.")
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

pub async fn error_parsing_json(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("Failed to parse the json file.")
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

pub async fn error_no_guild_id(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("Failed to get the guild id.")
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

pub async fn error_no_avatar(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("Failed to get avatar url.")
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

pub async fn error_no_module(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}", localised_text.forgot_module))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_no_token(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}", localised_text.no_token))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_no_base_url(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}", localised_text.no_base_url))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_message_followup(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &String,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(format!("{}", error_message))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_message_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &String,
    mut message: Message,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = message
            .edit(&ctx.http, |message| {
                message.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(format!("{}", error_message))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_no_token_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    mut message: Message,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found_edit(color, ctx, message.clone()).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file_edit(color, ctx, message.clone()).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json_edit(color, ctx, message.clone()).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id_edit(color, ctx, message.clone()).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = message
            .edit(&ctx.http, |message| {
                message.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(format!("{}", localised_text.no_token))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error_edit(color, ctx, message.clone()).await
    }
}

pub async fn error_no_base_url_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    mut message: Message,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found_edit(color, ctx, message.clone()).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file_edit(color, ctx, message.clone()).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json_edit(color, ctx, message.clone()).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id_edit(color, ctx, message.clone()).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = message
            .edit(&ctx.http, |message| {
                message.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(format!("{}", localised_text.no_base_url))
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error_edit(color, ctx, message.clone()).await
    }
}

pub async fn no_langage_error_edit(color: Colour, ctx: &Context, mut message: Message) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title("Error")
                    .description("Langage does not exist.")
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn error_no_guild_id_edit(color: Colour, ctx: &Context, mut message: Message) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title("Error")
                    .description("Failed to get the guild id.")
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn error_parsing_json_edit(color: Colour, ctx: &Context, mut message: Message) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title("Error")
                    .description("Failed to parse the json file.")
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn error_cant_read_file_edit(color: Colour, ctx: &Context, mut message: Message) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title("Error")
                    .description("The langage file can't be read.")
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn error_file_not_found_edit(color: Colour, ctx: &Context, mut message: Message) {
    if let Err(why) = message
        .edit(&ctx.http, |message| {
            message.embed(|m| {
                m.title("Error")
                    .description("The langage file was not found.")
                    .timestamp(Timestamp::now())
                    .color(color)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub async fn error_not_implemented(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(&localised_text.error_title)
                                .description(format!("{}", &localised_text.not_implemented))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error(color, ctx, command).await
    }
}

pub async fn error_making_request_edit(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    mut message: Message,
) {
    let mut file = match File::open("lang_file/embed/error.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, ErrorLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;
    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        if let Err(why) = message
            .edit(&ctx.http, |message| {
                message.embed(|m| {
                    m.title(&localised_text.error_title)
                        .description(&localised_text.error_request)
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        no_langage_error_edit(color, ctx, message)
    }
}
