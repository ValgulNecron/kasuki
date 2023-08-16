use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::RegisterLocalisedText;
use crate::cmd::general_module::pool::get_pool;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user (
            user_id TEXT PRIMARY KEY,
            anilist_username TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    let color = Colour::FABLED_PINK;

    if let CommandDataOptionValue::String(username) = option {
        let user_id = &command.user.id.to_string();
        let user_pfp_ref = &command.user.avatar.as_ref().unwrap();
        let profile_picture;
        if let Some(first) = user_pfp_ref.split('_').next() {
            if first == "a" {
                profile_picture = format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.gif?size=1024",
                    user_id, user_pfp_ref
                );
            } else {
                profile_picture = format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.webp?size=1024",
                    user_id, user_pfp_ref
                );
            }
        } else {
            profile_picture = format!(
                "https://cdn.discordapp.com/avatars//{}/{}.webp?size=1024",
                user_id, user_pfp_ref
            );
        }
        sqlx::query(
            "INSERT OR REPLACE INTO registered_user (user_id, anilist_username) VALUES (?, ?)",
        )
        .bind(user_id)
        .bind(username)
        .execute(&pool)
        .await
        .unwrap();

        let mut file =
            File::open("lang_file/embed/anilist/register.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, RegisterLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title(username)
                                    // Add a timestamp for the current time
                                    // This also accepts a rfc3339 Timestamp
                                    .timestamp(Timestamp::now())
                                    .thumbnail(profile_picture)
                                    .color(color)
                                    .description(format!(
                                        "{}{}{}{}{}",
                                        &localised_text.part_1,
                                        user_id,
                                        &localised_text.part_2,
                                        username,
                                        &localised_text.part_3
                                    ))
                            })
                        })
                })
                .await
            {
                println!("{}: {}", localised_text.error_slash_command, why);
            }
        } else {
            return "Language not found".to_string();
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("register")
        .description("Register your anilist username for ease of use.")
        .create_option(|option| {
            option
                .name("username")
                .description("Your anilist user name.")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
