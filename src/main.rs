use serenity::all::{ActivityData, Context, EventHandler, GatewayIntents, Interaction, Ready};
use serenity::{async_trait, Client};
use std::env;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

use struct_shard_manager::ShardManagerContainer;

use crate::activity::anime_activity::manage_activity;
use crate::command_autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command_register::command_registration::creates_commands;
use crate::command_run::command_dispatch::command_dispatching;
use crate::common::calculate_user_color::color_management;
use crate::components::components_dispatch::components_dispatching;
use crate::constant::ACTIVITY_NAME;
use crate::database::dispatcher::init_dispatch::init_sql_database;
use crate::game_struct::steam_game_id_struct::get_game;
use crate::logger::{create_log_directory, init_logger, remove_old_logs};

mod activity;
mod anilist_struct;
mod command_autocomplete;
mod command_register;
mod command_run;
mod common;
mod components;
mod constant;
mod database;
mod database_struct;
mod error_enum;
mod error_management;
mod game_struct;
mod image_saver;
mod lang_struct;
mod logger;
pub mod struct_shard_manager;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = ctx.cache.guilds();
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            info!("Launching the user color management thread!");
            color_management(guilds, ctx_clone).await;
        });

        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            info!("Launching the activity management thread!");
            manage_activity(ctx_clone).await
        });

        let activity_type = Some(ActivityData::custom(ACTIVITY_NAME));
        ctx.set_activity(activity_type);

        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        let server_number = &ctx.cache.guilds().len();
        info!(server_number);

        for guild in ctx.cache.guilds() {
            let partial_guild = guild.to_partial_guild(&ctx.http).await.unwrap();
            debug!(
                "guild name {} (guild id: {})",
                &partial_guild.name,
                &partial_guild.id.to_string()
            )
        }

        let my_path = ".env";
        let path = std::path::Path::new(my_path);
        let _ = dotenv::from_path(path);

        let remove_old_commmand = env::var("REMOVE_OLD_COMMAND")
            .unwrap_or("false".to_string())
            .to_lowercase()
            .as_str()
            == "true";

        trace!(remove_old_commmand);

        creates_commands(&ctx.http, remove_old_commmand).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command_interaction) = interaction.clone() {
            info!(
                "Received {} from {} in {}",
                command_interaction.data.name,
                command_interaction.user.name,
                command_interaction.guild_id.unwrap().to_string()
            );
            debug!(
                "Received command interaction: {}, Option: {:?}, User: {}({})",
                command_interaction.data.name,
                command_interaction.data.options,
                command_interaction.user.name,
                command_interaction.user.id
            );
            trace!("{:#?}", command_interaction);
            if let Err(e) = command_dispatching(ctx, command_interaction).await {
                error_management::error_dispatch::command_dispatching(e).await
            }
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            autocomplete_dispatching(ctx, autocomplete_interaction).await
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            if let Err(e) = components_dispatching(ctx, component_interaction).await {
                trace!("{:?}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Bot starting please wait.");
    // Configure the client with your Discord bot token in the environment.
    let my_path = ".env";
    let path = std::path::Path::new(my_path);
    let _ = dotenv::from_path(path);

    let env = env::var("RUST_LOG")
        .unwrap_or("info".to_string())
        .to_lowercase();
    let log = env.as_str();
    match create_log_directory() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    let _ = remove_old_logs().is_ok();

    match init_logger(log) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    match init_sql_database().await {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    }

    tokio::spawn(async move {
        info!("Launching the game management thread!");
        get_game().await
    });

    info!("starting the bot.");
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            debug!("Successfully got the token from env.");
            trace!(token);
            token
        }
        Err(_) => {
            error!("Env variable not set exiting.");
            return;
        }
    };

    // Build our client.
    let gateway_intent = GatewayIntents::GUILD_MEMBERS;
    let mut client = match Client::builder(token, gateway_intent)
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

        data.insert::<ShardManagerContainer>(Arc::clone(shard_manager))
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why)
    }
}
