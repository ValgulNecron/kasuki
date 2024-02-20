use crate::constant::{IMAGE_BASE_URL, TRANSCRIPT_MODELS};
use crate::constant::TRANSCRIPT_TOKEN;
use crate::constant::{CHAT_BASE_URL, CHAT_MODELS, CHAT_TOKEN, IMAGE_BASE_URL, IMAGE_MODELS, IMAGE_TOKEN, TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_SERVER_IMAGE_UPDATE, TIME_BETWEEN_USER_COLOR_UPDATE, TRANSCRIPT_BASE_URL};
use serenity::all::{ActivityData, Context, EventHandler, GatewayIntents, Interaction, Ready};
use serenity::all::{Guild, Member};
use serenity::{async_trait, Client};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, trace};

use struct_shard_manager::ShardManagerContainer;

use crate::activity::anime_activity::manage_activity;
use crate::command_autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command_register::command_registration::creates_commands;
use crate::command_run::command_dispatch::{check_if_module_is_on, command_dispatching};
use crate::components::components_dispatch::components_dispatching;
use crate::constant::{ACTIVITY_NAME, DELAY_BEFORE_THREAD_SPAWN, MAX_LOG_RETENTION_DAYS};
use crate::database::dispatcher::init_dispatch::init_sql_database;
use crate::error_management::error_dispatch;
use crate::game_struct::steam_game_id_struct::get_game;
use crate::logger::{create_log_directory, init_logger, remove_old_logs};
use crate::new_member::new_member;
use crate::server_image::calculate_user_color::color_management;
use crate::server_image::generate_server_image::server_image_management;
use crate::web_server::launcher::web_server_launcher;

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
mod error_management;
mod game_struct;
mod image_saver;
mod lang_struct;
mod logger;
mod new_member;
mod server_image;
pub mod struct_shard_manager;
mod web_server;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.unwrap_or_default() {
            color_management(&ctx.cache.guilds(), &ctx).await;
            server_image_management(&ctx).await;
            debug!("Joined a new guild: {} at {}", guild.name, guild.joined_at);
        } else {
            debug!("Got info from guild: {} at {}", guild.name, guild.joined_at);
        }
    }

    async fn guild_member_addition(&self, ctx: Context, mut member: Member) {
        color_management(&ctx.cache.guilds(), &ctx).await;
        server_image_management(&ctx).await;
        let guild_id = member.guild_id.to_string();
        trace!("Member {} joined guild {}", member.user.tag(), guild_id);
        if check_if_module_is_on(guild_id, "GAME").await.unwrap_or(true) {
            if let Err(e) = new_member(ctx, &mut member).await {
                error!("{:?}", e)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            thread_management_launcher(ctx_clone).await;
        });

        let activity_type = Some(ActivityData::custom(ACTIVITY_NAME.clone()));
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
            trace!("{:?}", command_interaction.user);
            trace!("{:?}", command_interaction.data);
            trace!("{:?}", command_interaction.guild_id);
            let command_version = command_interaction.version;
            trace!(command_version);
            if let Err(e) = command_dispatching(&ctx, &command_interaction).await {
                error_dispatch::command_dispatching(e, &command_interaction, &ctx).await
            }
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            autocomplete_dispatching(ctx, autocomplete_interaction).await
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            if let Err(e) = components_dispatching(ctx, component_interaction).await {
                error!("{:?}", e)
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

    unsafe {
        MAX_LOG_RETENTION_DAYS = env::var("MAX_LOG_RETENTION_DAYS")
            .unwrap_or("7".to_string())
            .parse()
            .unwrap_or(7);
    }
    tokio::spawn(async move {
        info!("Launching log management thread (the one that remove old one).");
        let _ = remove_old_logs().is_ok();
    });

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

    unsafe {
        trace!("{}", IMAGE_BASE_URL);
        trace!("{}",IMAGE_TOKEN);
        trace!("{}",IMAGE_MODELS);
        trace!("{}",CHAT_BASE_URL);
        trace!("{}",CHAT_TOKEN);
        trace!("{}",CHAT_MODELS);
        trace!("{}",TRANSCRIPT_BASE_URL);
        trace!("{}",TRANSCRIPT_TOKEN);
        trace!("{}",TRANSCRIPT_MODELS);
    }

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
    let gateway_intent_non_privileged = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_INVITES
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MODERATION
        | GatewayIntents::GUILD_SCHEDULED_EVENTS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_WEBHOOKS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_TYPING
        | GatewayIntents::AUTO_MODERATION_CONFIGURATION
        | GatewayIntents::AUTO_MODERATION_EXECUTION;
    let gateway_intent_privileged = GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS
        //         | GatewayIntents::MESSAGE_CONTENT
        ;
    let gateway_intent = gateway_intent_non_privileged | gateway_intent_privileged;
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

async fn thread_management_launcher(ctx: Context) {
    let guilds = ctx.cache.guilds();
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        loop {
            info!("Launching the user color management thread!");
            color_management(&guilds, &ctx_clone).await;
            sleep(Duration::from_secs(TIME_BETWEEN_USER_COLOR_UPDATE)).await;
        }
    });

    info!("Waiting 30second before launching the different thread.");
    sleep(Duration::from_secs(DELAY_BEFORE_THREAD_SPAWN)).await;

    tokio::spawn(async move {
        info!("Launching the log web server thread!");
        web_server_launcher().await
    });
    sleep(Duration::from_secs(5)).await;

    tokio::spawn(async move {
        info!("Launching the game management thread!");
        get_game().await
    });

    sleep(Duration::from_secs(5)).await;
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        info!("Launching the activity management thread!");
        manage_activity(ctx_clone).await
    });

    sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        loop {
            info!("Launching the server image management thread!");
            server_image_management(&ctx_clone).await;
            sleep(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE)).await;
        }
    });

    info!("Done spawning thread manager.");
}
