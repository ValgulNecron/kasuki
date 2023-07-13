extern crate core;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Activity;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::time::sleep;

use crate::cmd::ai_module::*;
use crate::cmd::anilist_module::*;
use crate::cmd::general_module::pool::get_pool;
use crate::cmd::general_module::struct_shard_manager::ShardManagerContainer;
use crate::cmd::general_module::*;

mod cmd;

struct Handler;

const ACTIVITY_NAME: &str = "Do /help to get the list of command";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Add activity to the bot as the type in activity_type and with ACTIVITY_NAME as name
        let activity_type: Activity = Activity::playing(ACTIVITY_NAME);
        ctx.set_activity(activity_type).await;

        println!("{} is connected!", ready.user.name);

        // Create all the commande found in cmd. (if a command is added it will need to be added here).
        let guild_command = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                // General module.
                .create_application_command(|command| ping::register(command))
                .create_application_command(|command| lang::register(command))
                .create_application_command(|command| info::register(command))
                // Anilist module.
                .create_application_command(|command| anime::register(command))
                .create_application_command(|command| character::register(command))
                .create_application_command(|command| compare::register(command))
                .create_application_command(|command| level::register(command))
                .create_application_command(|command| ln::register(command))
                .create_application_command(|command| manga::register(command))
                .create_application_command(|command| random::register(command))
                .create_application_command(|command| register::register(command))
                .create_application_command(|command| search::register(command))
                .create_application_command(|command| staff::register(command))
                .create_application_command(|command| user::register(command))
                // AI module.
                .create_application_command(|command| image::register(command))
                .create_application_command(|command| transcript::register(command))
                .create_application_command(|command| translation::register(command))
        })
        .await;

        println!(
            "I created the following global slash command: {:#?}",
            guild_command
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                // General module.
                "ping" => ping::run(&ctx, &command).await,
                "lang" => lang::run(&command.data.options, &ctx, &command).await,
                "info" => info::run(&ctx, &command).await,

                // Anilist module
                "anime" => anime::run(&command.data.options, &ctx, &command).await,
                "character" => character::run(&command.data.options, &ctx, &command).await,
                "compare" => compare::run(&command.data.options, &ctx, &command).await,
                "level" => level::run(&command.data.options, &ctx, &command).await,
                "ln" => ln::run(&command.data.options, &ctx, &command).await,
                "manga" => manga::run(&command.data.options, &ctx, &command).await,
                "random" => random::run(&command.data.options, &ctx, &command).await,
                "register" => register::run(&command.data.options, &ctx, &command).await,
                "search" => search::run(&command.data.options, &ctx, &command).await,
                "staff" => staff::run(&command.data.options, &ctx, &command).await,
                "user" => user::run(&command.data.options, &ctx, &command).await,

                // AI module
                "image" => image::run(&command.data.options, &ctx, &command).await,
                "transcript" => transcript::run(&command.data.options, &ctx, &command).await,
                "translation" => translation::run(&command.data.options, &ctx, &command).await,

                _ => "not implemented :(".to_string(),
            };

            // check if the command was successfully done.
            if content == "good".to_string() {
                return;
            }
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let my_path = "./.env";
    println!("{}", my_path.to_string());
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);
    let token = env::var("DISCORD_TOKEN").expect("discord token");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(600)).await;

            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;

            let database_url = "./data.db";
            let pool = get_pool(database_url).await;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS ping_history (
                        shard_id TEXT,
                        timestamp TEXT,
                        ping TEXT NOT NULL,
                        PRIMARY KEY (shard_id, timestamp)
                    )",
            )
            .execute(&pool)
            .await
            .unwrap();

            for (id, runner) in shard_runners.iter() {
                let shard_id = id.0.to_string();
                let latency_content = runner.latency.unwrap_or(Duration::from_secs(0));
                let latency = format!("{:?}", latency_content);
                let now = Utc::now().timestamp().to_string();
                sqlx::query(
                    "INSERT OR REPLACE INTO ping_history (shard_id, timestamp, ping) VALUES (?, ?, ?)",
                )
                    .bind(shard_id)
                    .bind(now)
                    .bind(latency)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
        }
    });

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {:?}", why);
    }
}
