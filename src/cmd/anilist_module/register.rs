use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use sqlx::{Row, SqlitePool};

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let database_url = "./data.db";
    let pool = match SqlitePool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            return "Error: Failed to connect to the database.".to_string();
        }
    };

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user (
            user_id TEXT PRIMARY KEY,
            anilist_username TEXT NOT NULL
        )",
    )
        .execute(&pool)
        .await.unwrap();
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    let color = Colour::FABLED_PINK;

    if let CommandDataOptionValue::String(username) = option {
        let user_id = &command.user.id.to_string();
        let profile_picture = format!("https://cdn.discordapp.com/avatars/{}", &command.user.avatar.as_ref().unwrap());
        sqlx::query("INSERT OR REPLACE INTO registered_user (user_id, anilist_username) VALUES (?, ?)")
            .bind(user_id)
            .bind(username)
            .execute(&pool)
            .await.unwrap();
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(username)
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(Timestamp::now())
                                .image(profile_picture)
                                .color(color)
                                .description(format!("The user {} was linked to {} anilist", user_id, username))
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("register").description("Register your anilist username for ease of use.").create_option(
        |option| {
            option
                .name("username")
                .description("Your anilist user name.")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}