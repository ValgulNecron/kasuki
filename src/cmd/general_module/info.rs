use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::error_handling::{
    error_cant_read_file, error_file_not_found, error_no_guild_id, error_parsing_json,
    no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::InfoLocalisedText;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    let mut file = match File::open("lang_file/embed/general/profile.json") {
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

    let json_data: HashMap<String, InfoLocalisedText> = match serde_json::from_str(&json) {
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
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(&localised_text.title)
                                .description(&localised_text.description)
                                .footer(|f| f.text(&localised_text.footer))
                                .timestamp(Timestamp::now())
                                .color(color)
                        })
                        .components(|components| {
                            components.create_action_row(|row| {
                                row.create_button(|button| {
                                    button.label(&localised_text.button_see_on_github)
                                        .url("https://github.com/ValgulNecron/DIscordAnilistBotRS")
                                        .style(ButtonStyle::Link)
                                })
                                    .create_button(|button| {
                                        button.label(&localised_text.button_official_website)
                                            .url("https://kasuki.valgul.moe/")
                                            .style(ButtonStyle::Link)
                                    })
                            })
                                .create_action_row(|button| {
                                    button.create_button(|button| {
                                        button.label(&localised_text.button_official_discord)
                                            .url("https://discord.gg/dWGU6mkw7J")
                                            .style(ButtonStyle::Link)
                                    })
                                        .create_button(|button| {
                                            button.label(&localised_text.button_add_the_bot)
                                                .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot")
                                                .style(ButtonStyle::Link)
                                        })
                                })
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    } else {
        let color = Colour::FABLED_PINK;
        no_langage_error(color, ctx, command).await
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("info").description("bot info")
}
