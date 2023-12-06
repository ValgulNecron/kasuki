mod anilist_struct;
mod command_autocomplete;
mod command_register;
mod command_run;
mod common;
mod constant;
mod error_enum;
mod error_management;
mod lang_struct;
mod logger;
mod sqls;

use crate::command_autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command_register::command_registration::creates_commands;
use crate::command_run::command_dispatch::command_dispatching;
use crate::constant::ACTIVITY_NAME;
use crate::lang_struct::struct_shard_manager::ShardManagerContainer;
use crate::logger::{create_log_directory, init_logger, remove_old_logs};
use crate::sqls::general::sql::init_sql_database;
use serenity::all::{ActivityData, Context, EventHandler, GatewayIntents, Interaction, Ready};
use serenity::{async_trait, Client};
use std::env;
use std::sync::Arc;
use tracing::{debug, error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Add activity to the bot as the type in activity_type and with ACTIVITY_NAME as name
        let activity_type = Some(ActivityData::custom(ACTIVITY_NAME));
        ctx.set_activity(activity_type);

        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        creates_commands(&ctx.http).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction.clone() {
            info!(
                "Received command interaction: {}, Option: {:#?}, User: {}({})",
                command.data.name, command.data.options, command.user.name, command.user.id
            );
            match command_dispatching(ctx, command).await {
                Err(e) => error_management::error_dispatch::command_dispatching(e).await,
                _ => {}
            };

            // check if the command was successfully done.
        } else if let Interaction::Autocomplete(command) = interaction.clone() {
            autocomplete_dispatching(ctx, command).await;
        }
    }
}

#[tokio::main]
async fn main() {
    error!("{:?}", init_sql_database().await);
    // Configure the client with your Discord bot token in the environment.
    let my_path = "./.env";
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);
    let env = env::var("RUST_LOG")
        .unwrap_or("info".to_string())
        .to_lowercase();
    let log = env.as_str();
    match create_log_directory() {
        Ok(_) => {}
        Err(_) => return,
    };

    let _ = remove_old_logs().is_ok();

    match init_logger(log) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    info!("starting the bot.");
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            debug!("Successfully got the token from env.");
            token
        }
        Err(_) => {
            error!("Env variable not set exiting.");
            return;
        }
    };

    // Build our client.
    let mut client = match Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
    {
        Ok(client) => {
            debug!("Client created.");
            client
        }
        Err(e) => {
            error!("Error while creating client: {}", e);
            return;
        }
    };

    {
        let mut data = client.data.write().await;

        let shard_manager = &client.shard_manager;

        data.insert::<ShardManagerContainer>(Arc::clone(shard_manager));
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
