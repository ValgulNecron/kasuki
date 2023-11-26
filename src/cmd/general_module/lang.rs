use crate::available_lang::AvailableLang;
use crate::error_enum::AppError::{LangageGuildIdError, NoCommandOption};
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::function::error_management::no_lang_error::error_no_langage_guild_id;
use crate::function::sqls::general::data::set_data_guild_langage;
use crate::structure::embed::general::struct_lang_lang::LangLocalisedText;
use crate::structure::register::general::struct_lang_register::LangRegister;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::application_command::CommandDataOptionValue;
use serenity::model::{Permissions, Timestamp};
use serenity::utils::Colour;
use std::any::Any;
use std::f32::consts::E;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    let option = options
        .get(0)
        .expect("Expected lang option")
        .resolved
        .as_ref()
        .expect("Expected lang object");
    let lang = options
        .get(0)
        .ok_or(|_| OPTION_ERROR)?
        .resolved
        .ok_or(|_| OPTION_ERROR)?;
    let lang = match lang {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no otpion.",
            )))
        }
    };
    let color = Colour::FABLED_PINK;

    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    set_data_guild_langage(guild_id, &lang).await;
    let localised_text = match LangLocalisedText::get_ping_localised(color, ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(&localised_text.title)
                            .description(format!("{}{}", &localised_text.description, lang))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let available_langages = AvailableLang::get_available_lang().unwrap();
    let langages = LangRegister::get_profile_register_localised().unwrap();
    command
        .name("lang")
        .description("Change the lang of the bot response")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .create_option(|option| {
            option
                .name("lang")
                .description("The lang you want to set the response to.")
                .kind(CommandOptionType::String)
                .required(true);
            for langages in available_langages.values() {
                option.add_string_choice(&langages.lang, &langages.lang);
            }
            for lang in langages.values() {
                option
                    .name_localized(&lang.code, &lang.option1)
                    .description_localized(&lang.code, &lang.option1_desc);
            }
            option
        });
    for lang in langages.values() {
        command
            .name_localized(&lang.code, &lang.option1)
            .description_localized(&lang.code, &lang.option1_desc);
    }
    command
}
