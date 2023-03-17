use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use crate::client::DB;
use crate::models::Model;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(user) = option {
        let user_id = command.user.id.as_u64().to_string();
        let anilist_username = user.clone();
        connect("surrealdb.valgul.moe", "discord_bot", "discord_bot");
        if !conn.table_exists("User") {
            conn.execute("CREATE TABLE User (discord_id TEXT PRIMARY KEY, anilist_username TEXT)").unwrap();
        }
        let sql = "INSERT OR REPLACE INTO User (discord_id, anilist_username) VALUES (?, ?)";
        conn.execute_with_params(sql, &[&user_id, &anilist_username]).unwrap();
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title("Registration Successful")
                                .description(format!("{} registered {} successfully", user_id, anilist_username))
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
    command.name("register").description("Register an anilist username to the default for the user command").create_option(
        |option| {
            option
                .name("username")
                .description("The username of the anilist user you want to register as your account")
                .kind(CommandOptionType::String)
                .required(true)
        },
    )
}

pub static DB: Surreal<surrealdb::engine::local::Db> = Surreal::init();

use surrealdb::Surreal;

use crate::errors::ApiResult;

pub async fn connect(address: &str, namespace: &str, database: &str) -> ApiResult<()> {
  DB.connect::<surrealdb::engine::local::File>(address)
    .await?;

  DB.use_ns(namespace).use_db(database).await?;

  Ok(())
}