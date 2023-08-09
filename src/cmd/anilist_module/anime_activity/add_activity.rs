use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::autocomplete::AutocompleteInteraction;

use crate::cmd::anilist_module::struct_autocomplete_media::MediaPageWrapper;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::AddActivityLocalisedText;
use crate::cmd::general_module::pool::get_pool;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let database_url = "./data.db";
    let pool = get_pool(database_url);

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_data    (
        anime_id TEXT,
        timestamp TEXT,
        server_id TEXT,
        webhook TEXT,
        PRIMARY KEY (anime_id, server_id)
    )",
    )
        .execute(&pool)
        .await
        .unwrap();

    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let mut file = File::open("lang_file/anilist/user.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, AddActivityLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let anime_id;
            let channel_id = command.channel_id.0;
            let map = json!({"name": });
            let webhook = ctx.http.create_webhook(channel_id
                                                  ,
                                                  , None)
            sqlx::query(
                "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook) VALUES (?, ?, ?, ?)",
            )
                .bind()
                .bind()
                .bind()
                .bind()
                .execute(&pool)
                .await
                .unwrap();
        }
    }
    "good".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_activity")
        .description("Add an anime activity")
        .create_option(|option| {
            option
                .name("anime_name")
                .description("Name of the anime you want to add as an activity")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_anime(search, 8, "ANIME").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}