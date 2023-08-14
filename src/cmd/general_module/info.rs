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

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::InfoLocalisedText;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let color = Colour::FABLED_PINK;

    let mut file = File::open("lang_file/general/info.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, InfoLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
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
        return "Language not found".to_string();
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("info").description("bot info")
}
