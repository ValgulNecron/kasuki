use crate::cache::manage::cache_init::init_cache;
use serenity::all::{
    GatewayIntents,
    ShardManager,
};
use serenity::{Client};
use std::env;
use std::sync::Arc;
use struct_shard_manager::ShardManagerContainer;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::constant::{
    APP_TUI, DISCORD_TOKEN,
};
use crate::database::manage::dispatcher::init_dispatch::init_sql_database;
use crate::event_handler::{Handler, RootUsage};
use crate::logger::{create_log_directory, init_logger};

mod api;
mod background_task;
mod cache;
mod command;
mod command_register;
mod components;
pub(crate) mod constant;
mod database;
pub(crate) mod event_handler;
mod grpc_server;
mod helper;
mod logger;
mod new_member;
mod struct_shard_manager;
mod structure;
mod tui;

#[tokio::main]
/// The main function where the execution of the bot starts.
/// It initializes the logger, the SQL database, and the bot client.
/// It also spawns asynchronous tasks for managing the ping of the shards and starting the client.
async fn main() {
    let _ = dotenvy::from_path(".env");

    // Print a message indicating the bot is starting.
    println!("Bot starting please wait.");
    // Load environment variables from the .env file.

    // Get the log level from the environment variable "RUST_LOG".
    // If the variable is not set, default to "info".
    let log = env::var("RUST_LOG")
        .unwrap_or("info".to_string())
        .to_lowercase();
    let log = log.as_str();

    // Create the log directory.
    // If an error occurs, print the error and return.
    if let Err(e) = create_log_directory() {
        eprintln!("{:?}", e);
        return;
    }

    // Initialize the logger with the specified log level.
    // If an error occurs, print the error and return.
    if let Err(e) = init_logger(log) {
        eprintln!("{:?}", e);
        return;
    }

    let app_tui = APP_TUI;
    if *app_tui {
        // create a new tui in a new thread
        tokio::spawn(async {
            tui::create_tui().await.unwrap();
        });
    }

    // Initialize the SQL database.
    // If an error occurs, log the error and return.
    if let Err(e) = init_sql_database().await {
        error!("{:?}", e);
        return;
    }
    if let Err(e) = init_cache().await {
        error!("{:?}", e);
        return;
    }

    // Log a message indicating the bot is starting.
    info!("starting the bot.");

    let number_of_command_use = Arc::new(RwLock::new(0u128));
    let number_of_command_use_per_command: RootUsage;
    // populate the number_of_command_use_per_command with the content of the file
    if let Ok(content) = std::fs::read_to_string("command_use.json") {
        number_of_command_use_per_command =
            serde_json::from_str(&content).unwrap_or_else(|_| RootUsage::new());
    } else {
        number_of_command_use_per_command = RootUsage::new();
    }
    let number_of_command_use_per_command =
        Arc::new(RwLock::new(number_of_command_use_per_command));
    let handler = Handler {
        number_of_command_use,
        number_of_command_use_per_command,
    };

    // Get all the non-privileged intent.
    let gateway_intent_non_privileged = GatewayIntents::non_privileged();
    // Get the needed privileged intent.
    let gateway_intent_privileged = GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MEMBERS
        //         | GatewayIntents::MESSAGE_CONTENT
        ;
    // Combine both intents for the client to consume.
    let gateway_intent = gateway_intent_non_privileged | gateway_intent_privileged;
    // Create a new client instance using the provided token and gateway intents.
    // The client is built with an event handler of type `Handler`.
    // If the client creation fails, log the error and exit the process.
    let discord_token = DISCORD_TOKEN;
    let discord_token = discord_token.as_str();
    let mut client = Client::builder(discord_token, gateway_intent)
        .event_handler(handler)
        .await
        .unwrap_or_else(|e| {
            error!("Error while creating client: {}", e);
            std::process::exit(1);
        });

    // Clone the shard manager from the client.
    let shard_manager = client.shard_manager.clone();
    let shutdown = shard_manager.clone();
    // Insert the cloned shard manager into the client's data.
    // This allows for the shard manager to be accessed from the context in event handlers.
    client
        .data
        .write()
        .await
        .insert::<ShardManagerContainer>(Arc::clone(&shard_manager));

    // Spawn a new asynchronous task for starting the client.
    // If the client fails to start, log the error.
    tokio::spawn(async move {
        if let Err(why) = client.start_autosharded().await {
            error!("Client error: {:?}", why)
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
        ShardManager::shutdown_all(&shutdown).await;
        info!("Received bot shutdown signal. Shutting down bot.");
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
        ShardManager::shutdown_all(&shutdown).await;
        info!("Received bot shutdown signal. Shutting down bot.");
    }
}