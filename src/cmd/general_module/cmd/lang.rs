use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::{Permissions, Timestamp};
use serenity::utils::Colour;

use crate::cmd::error_module::no_lang_error::error_no_langage_guild_id;
use crate::cmd::general_module::function::pool::get_pool;
use crate::cmd::lang_struct::available_lang::AvailableLang;
use crate::cmd::lang_struct::embed::general::struct_lang_lang::LangLocalisedText;
use crate::cmd::lang_struct::register::general::struct_lang_register::LangRegister;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let option = options
        .get(0)
        .expect("Expected lang option")
        .resolved
        .as_ref()
        .expect("Expected lang object");
    let color = Colour::FABLED_PINK;

    if let CommandDataOptionValue::String(lang) = option {
        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(color, ctx, command).await;
                return;
            }
        };
        let localised_text = match LangLocalisedText::get_ping_localised(color, ctx, command).await
        {
            Ok(data) => data,
            Err(_) => return,
        };
        sqlx::query("INSERT OR REPLACE INTO guild_lang (guild, lang) VALUES (?, ?)")
            .bind(guild_id)
            .bind(lang)
            .execute(&pool)
            .await
            .unwrap();

        if let Err(why) = command
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
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
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
