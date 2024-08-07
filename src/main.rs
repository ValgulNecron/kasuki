use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use serenity::all::{GatewayIntents, ShardManager};
use serenity::Client;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::Config;
use crate::constant::{CACHE_MAX_CAPACITY, COMMAND_USE_PATH, TIME_BETWEEN_CACHE_UPDATE};
use crate::database::manage::dispatcher::init_dispatch::init_sql_database;
use crate::event_handler::{BotData, Handler, RootUsage};
use crate::logger::{create_log_directory, init_logger};
use crate::struct_shard_manager::ShardManagerContainer;

mod background_task;
mod command;
mod command_register;
mod components;
mod config;
mod constant;
mod custom_serenity_impl;
mod database;
mod event_handler;
mod grpc_server;
mod helper;
mod logger;
mod new_member;
mod removed_member;
mod struct_shard_manager;
mod structure;
mod audio;

#[tokio::main]
/// The main function where the execution of the bot starts.
/// It initializes the logger, the SQL database, and the bot client.
/// It also spawns asynchronous tasks for managing the ping of the shards and starting the client.
async fn main() {
    println!("Preparing bot environment please wait...");
    // read config.toml as string
    let config = match std::fs::read_to_string("config.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error while reading config.toml: {:?}", e);
            std::process::exit(1);
        }
    };
    let mut config: Config = match toml::from_str(&config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error while parsing config.toml: {:?}", e);
            std::process::exit(1);
        }
    };
    config.set_default_value_on_none();
    let log = config.logging.log_level.clone();
    let discord_token = config.bot.discord_token.clone();
    let db_type = config.bot.config.db_type.clone();
    let db_type = db_type.as_str();
    let max_log_retention_days = config.logging.max_log_retention;
    let config = Arc::new(config);

    // Get the log level from the environment variable "RUST_LOG".
    // If the variable is not set, default to "info".
    let log = log.as_str();

    // Create the log directory.
    // If an error occurs, print the error and return.
    if let Err(e) = create_log_directory() {
        eprintln!("{:?}", e);
        std::process::exit(2);
    }

    // Initialize the logger with the specified log level.
    // If an error occurs, print the error and return.
    if let Err(e) = init_logger(log, max_log_retention_days) {
        eprintln!("{:?}", e);
        std::process::exit(2);
    }

    // Initialize the SQL database.
    // If an error occurs, log the error and return.
    if let Err(e) = init_sql_database(db_type).await {
        error!("{:?}", e);
        std::process::exit(4);
    }

    let number_of_command_use_per_command: RootUsage;
    // populate the number_of_command_use_per_command with the content of the file
    if let Ok(content) = std::fs::read_to_string(COMMAND_USE_PATH) {
        number_of_command_use_per_command =
            serde_json::from_str(&content).unwrap_or_else(|_| RootUsage::new());
    } else {
        number_of_command_use_per_command = RootUsage::new();
    }

    let number_of_command_use_per_command =
        Arc::new(RwLock::new(number_of_command_use_per_command));
    let cache: Cache<String, String> = Cache::builder()
        .time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build();
    let anilist_cache: Arc<RwLock<Cache<String, String>>> = Arc::new(RwLock::new(cache));
    let cache: Cache<String, String> = Cache::builder()
        .time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build();
    let vndb_cache: Arc<RwLock<Cache<String, String>>> = Arc::new(RwLock::new(cache));
    let bot_data: Arc<BotData> = Arc::new(BotData {
        number_of_command_use_per_command,
        config,
        bot_info: Arc::new(RwLock::new(None)),
        anilist_cache,
        vndb_cache,
        already_launched: false.into(),
        apps: Arc::new(Default::default()),
        user_blacklist_server_image: Arc::new(Default::default()),
    });
    let handler = Handler { bot_data };

    // Get all the non-privileged intent.
    let gateway_intent_non_privileged = GatewayIntents::non_privileged();
    // Get the needed privileged intent.
    let gateway_intent_privileged = GatewayIntents::GUILD_MEMBERS
        // | GatewayIntents::GUILD_PRESENCES
        //         | GatewayIntents::MESSAGE_CONTENT
        ;
    // Combine both intents for the client to consume.
    let mut intent = gateway_intent_non_privileged;
    intent |= gateway_intent_privileged;
    let gateway_intent = intent;

    // Log a message indicating the bot is starting.
    info!("Finished preparing the environment. Starting the bot.");

    // Create a new client instance using the provided token and gateway intents.
    // The client is built with an event handler of type `Handler`.
    // If the client creation fails, log the error and exit the process.
    let discord_token = discord_token.as_str();
    let mut client = Client::builder(discord_token, gateway_intent)
        .event_handler(handler)
        .await
        .unwrap_or_else(|e| {
            error!("Error while creating client: {}", e);
            std::process::exit(5);
        });
    let shard_manager = client.shard_manager.clone();
    client
        .data
        .write()
        .await
        .insert::<ShardManagerContainer>(Arc::clone(&shard_manager));

    // Clone the shard manager from the client.
    let shard_manager = client.shard_manager.clone();
    let shutdown = shard_manager.clone();

    // Spawn a new asynchronous task for starting the client.
    // If the client fails to start, log the error.
    tokio::spawn(async move {
        if let Err(why) = client.start_autosharded().await {
            error!("Client error: {:?}", why);
            std::process::exit(6);
        }
    });

    #[cfg(unix)]
    {
        // Create a signal handler for "all" signals in unix.
        // If a signal is received, print a shutdown message.
        // All signals and not only ctrl-c
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let mut sigquit =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()).unwrap();
        let mut sigusr1 =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();
        let mut sigusr2 =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined2()).unwrap();
        tokio::select! {
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
            _ = sigquit.recv() => {},
            _ = sigusr1.recv() => {},
            _ = sigusr2.recv() => {},
        }
        info!("Received bot shutdown signal. Shutting down bot.");
        ShardManager::shutdown_all(&shutdown).await;
        std::process::exit(0);
    }
    #[cfg(windows)]
    {
        // Create a signal handler for "all" signals in windows.
        // If a signal is received, print a shutdown message.
        // All signals and not only ctrl-c
        let mut ctrl_break = tokio::signal::windows::ctrl_break().unwrap();
        let mut ctrl_c = tokio::signal::windows::ctrl_c().unwrap();
        let mut ctrl_close = tokio::signal::windows::ctrl_close().unwrap();
        let mut ctrl_logoff = tokio::signal::windows::ctrl_logoff().unwrap();
        let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown().unwrap();
        tokio::select! {
            _ = ctrl_break.recv() => {},
            _ = ctrl_c.recv() => {},
            _ = ctrl_close.recv() => {},
            _ = ctrl_logoff.recv() => {},
            _ = ctrl_shutdown.recv() => {},
        }
        info!("Received bot shutdown signal. Shutting down bot.");
        ShardManager::shutdown_all(&shutdown).await;
        std::process::exit(0);
    }
}
