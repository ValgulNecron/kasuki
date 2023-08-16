use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::{Permissions, Timestamp};
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::InteractionResponseType;
use serenity::utils::Colour;
use sqlx::{Pool, Sqlite};

use crate::cmd::general_module::error_handling::error_no_module;
use crate::cmd::general_module::pool::get_pool;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_activation (
            guild_id TEXT PRIMARY KEY,
            ai_module INTEGER,
            anilist_module INTEGER
        )",
    )
        .execute(&pool)
        .await
        .unwrap();

    let color = Colour::FABLED_PINK;

    let mut module = "".to_string();
    let mut state = false;
    for option in options {
        if option.name == "module_name" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(module_option) = resolved {
                module = module_option.clone()
            } else {
                module = "".to_string();
            }
        }
        if option.name == "state" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::Boolean(state_option) = resolved {
                state = state_option.clone()
            } else {
                state = false
            }
        }
    }

    let guild_id = command.guild_id.unwrap().0.to_string().clone();

    match module.as_str() {
        "ANIME" => {
            let row = make_sql_request(guild_id.clone(), &pool).await;
            let (_, ai_module, _): (Option<String>, Option<bool>, Option<bool>) = row;

            let ai_value = match ai_module {
                Some(true) => 1,
                Some(false) => 0,
                None => 1,
            };

            sqlx::query(
                "INSERT OR REPLACE INTO module_activation (guild_id, anilist_module, ai_module) VALUES (?, ?, ?)",
            )
                .bind(&guild_id)
                .bind(state)
                .bind(ai_value)
                .execute(&pool)
                .await
                .unwrap();

            let text;
            if state {
                text = format!("The module was activated on the server");
            } else {
                text = format!("The module was deactivated on the server");
            }

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title("ANILIST")
                                    // Add a timestamp for the current time
                                    // This also accepts a rfc3339 Timestamp
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .description(text)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", "Error creating slash", why);
            }
        }
        "AI" => {
            let row = make_sql_request(guild_id.clone(), &pool).await;
            let (_, _, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;

            let anilist_value = match anilist_module {
                Some(true) => 1,
                Some(false) => 0,
                None => 1,
            };

            sqlx::query(
                "INSERT OR REPLACE INTO module_activation (guild_id, ai_module, anilist_module) VALUES (?, ?, ?)",
            )
                .bind(&guild_id)
                .bind(state)
                .bind(anilist_value)
                .execute(&pool)
                .await
                .unwrap();

            let text;
            if state {
                text = format!("The module was activated on the server");
            } else {
                text = format!("The module was deactivated on the server");
            }

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.embed(|m| {
                                m.title("AI")
                                    // Add a timestamp for the current time
                                    // This also accepts a rfc3339 Timestamp
                                    .timestamp(Timestamp::now())
                                    .color(color)
                                    .description(text)
                            })
                        })
                })
                .await
            {
                println!("{}: {}", "Error creating slash", why);
            }
        }
        _ => error_no_module(color, ctx, command).await,
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("module")
        .description("Turn on and of module.")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .create_option(|option| {
            option
                .name("module_name")
                .description("The name of the module you want to turn on or off")
                .kind(CommandOptionType::String)
                .add_string_choice("AI", "AI")
                .add_string_choice("ANIME", "ANIME")
                .required(true)
        })
        .create_option(|option| {
            option
                .name("state")
                .description("ON or OFF")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
}

pub async fn check_activation_status(module: String, guild_id: String) -> bool {
    let database_url = "./data.db";
    let pool = get_pool(database_url).await;

    let row: (Option<String>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module FROM module_activation WHERE guild_id = ?",
    )
        .bind(&guild_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None));

    let (_, ai_module, anilist_module): (Option<String>, Option<bool>, Option<bool>) = row;
    return match module.as_str() {
        "ANILIST" => anilist_module.unwrap_or(true),
        "AI" => ai_module.unwrap_or(true),
        _ => false,
    };
}

pub async fn make_sql_request(
    guild_id: String,
    pool: &Pool<Sqlite>,
) -> (Option<String>, Option<bool>, Option<bool>) {
    let row: (Option<String>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module FROM module_activation WHERE guild = ?",
    )
        .bind(&guild_id)
        .fetch_one(pool)
        .await
        .unwrap_or((None, None, None));
    row
}
