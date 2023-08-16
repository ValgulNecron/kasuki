use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serenity::client::Context;
use serenity::Error;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::ErrorLocalisedText;

pub async fn error_message(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction, error_message: &String) {
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
        no_langage_error(color, ctx, command)
    }
}

pub async fn error_followup_message() {

}

pub async fn error_message_with_a_message() {

}

pub async fn no_langage_error(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title("Error")
                            .description("Langage does not exist")
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

pub async fn error_message_with_why(color: Colour, ctx: &Context, command: &ApplicationCommandInteraction, error_message: &String, why: Error) {
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
        no_langage_error(color, ctx, command)
    }
}