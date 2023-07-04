use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;
use serenity::utils::Colour;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use crate::cmd::general_module::pool::get_pool;
use crate::cmd::general_module::lang_struct::LangLocalisedText;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let database_url = "./data.db";
    let pool =  get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
        )",
    ).execute(&pool)
        .await.unwrap();

    let option = options
        .get(0)
        .expect("Expected lang option")
        .resolved
        .as_ref()
        .expect("Expected lang object");
    let color = Colour::FABLED_PINK;

    if let CommandDataOptionValue::String(lang) = option {
        let guild_id = command.guild_id.unwrap().0.to_string();
        sqlx::query("INSERT OR REPLACE INTO guild_lang (guild, lang) VALUES (?, ?)")
            .bind(guild_id)
            .bind(lang)
            .execute(&pool)
            .await.unwrap();

        let mut file = File::open("lang_file/general/lang_lang.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, LangLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        if let Some(localised_text) = json_data.get(lang) {
        println!("{:?}", localised_text);
        if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title(&localised_text.title)
                            .description(format!("{}{}", &localised_text.description, lang))
                            .footer(|f| f.text(&localised_text.footer))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
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
    }

    return "good".to_string()
}


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("lang").description("Change the lang of the bot response").create_option(|option| {
        option
            .name("lang")
            .description("The lang you want to set the response to.")
            .kind(CommandOptionType::String)
            .add_string_choice("En", "En")
            .add_string_choice("Fr", "Fr")
            .add_string_choice("De", "De")
            .add_string_choice("Jp", "Jp")
            .required(true)
    })
}

