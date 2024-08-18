use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serde_json::Value;
use serenity::all::Context;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info};

use crate::api::grpc_server::launcher::grpc_server_launcher;
use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::background_task::update_random_stats::update_random_stats_launcher;
use crate::config::{BotConfigDetails, Config, ImageConfig};
use crate::constant::{
    TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_ACTIVITY_CHECK, TIME_BETWEEN_BLACKLISTED_USER_UPDATE,
    TIME_BETWEEN_BOT_INFO, TIME_BETWEEN_GAME_UPDATE, TIME_BETWEEN_PING_UPDATE,
    TIME_BETWEEN_SERVER_IMAGE_UPDATE, TIME_BETWEEN_USER_COLOR_UPDATE,
};
use crate::event_handler::{BotData, RootUsage};
use crate::{api, get_url};
use crate::structure::database::ping_history::ActiveModel;
use crate::structure::database::prelude::PingHistory;
use crate::structure::steam_game_id_struct::get_game;
use crate::type_map_key::ShardManagerContainer;

/// This function is responsible for launching various background threads that manage the bot's activity, games, and web server.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance used to access the bot's data and cache.
/// * `bot_data` - An `Arc` wrapped `BotData` instance containing the bot's configuration and data.
///
pub async fn thread_management_launcher(
    ctx: Context,
    bot_data: Arc<BotData>,
    db_config: BotConfigDetails,
) {
    // Clone the necessary data from bot_data
    let command_usage = bot_data.number_of_command_use_per_command.clone();
    let is_grpc_on = bot_data.config.grpc.grpc_is_on;
    let config = bot_data.config.clone();
    let anilist_cache = bot_data.anilist_cache.clone();
    let apps = bot_data.apps.clone();
    let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();

    // Spawn background threads to manage the bot's activity, games, and web server
    tokio::spawn(launch_user_color_management_thread(
        ctx.clone(),
        user_blacklist_server_image.clone(),
        db_config.clone(),
    ));
    tokio::spawn(launch_activity_management_thread(
        ctx.clone(),
        anilist_cache.clone(),
        db_config.clone(),
    ));
    tokio::spawn(launch_game_management_thread(apps));
    tokio::spawn(ping_manager_thread(ctx.clone(), db_config.clone()));
    tokio::spawn(update_user_blacklist(user_blacklist_server_image));
    tokio::spawn(update_random_stats_launcher(anilist_cache.clone()));
    tokio::spawn(update_bot_info(ctx.clone(), bot_data.clone()));

    // Wait for 1 second before launching the web server thread
    sleep(Duration::from_secs(1)).await;
    tokio::spawn(launch_web_server_thread(
        ctx.clone(),
        command_usage,
        is_grpc_on,
        config,
        bot_data.clone(),
    ));

    // Wait for a certain amount of time before launching the server image management thread
    sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;
    let image_config = bot_data.config.image.clone();
    tokio::spawn(launch_server_image_management_thread(
        ctx.clone(),
        image_config,
        db_config.clone(),
    ));

    // Log a message indicating that all threads have been launched
    info!("Done spawning thread manager.");
}

/// Asynchronously manages the ping thread.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance used to access the bot's data and cache.
/// * `db_type` - A `String` representing the type of database.
///
async fn ping_manager_thread(ctx: Context, db_config: BotConfigDetails) {
    // Log a message indicating that the ping thread is being launched
    info!("Launching the ping thread!");

    // Read the data from the context
    let data_read = ctx.data.read().await;

    // Get the ShardManager from the data
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return;
        }
    };

    // Define an interval for periodic updates
    let mut interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_PING_UPDATE));

    // Main loop for managing pings
    let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
        Ok(connection) => connection,
        Err(e) => {
            error!("Failed to connect to the database. {:#?}", e);
            return;
        }
    };
    loop {
        // Wait for the next interval tick
        interval.tick().await;

        // Lock the shard manager and iterate over the runners
        let runner = shard_manager.runners.lock().await;
        for (shard_id, shard) in runner.iter() {
            // Extract latency and current timestamp information
            let latency = shard.latency.unwrap_or_default().as_millis().to_string();
            let now = chrono::Utc::now().naive_utc();
            match PingHistory::insert(ActiveModel {
                shard_id: Set(shard_id.to_string()),
                latency: Set(latency),
                timestamp: Set(now),
                ..Default::default()
            })
            .exec(&connection)
            .await
            {
                Ok(_) => {
                    debug!("Updated ping history for shard {}.", shard_id);
                }
                Err(e) => {
                    error!(
                        "Failed to update ping history for shard {}. {:#?}",
                        shard_id, e
                    );
                }
            }
        }
    }
}

/// Asynchronously launches the web server thread.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance used to access the bot's data and cache.
/// * `command_usage` - An `Arc` wrapped `RwLock` containing the root command usage.
/// * `is_grpc_on` - A boolean indicating if GRPC is enabled.
/// * `config` - An `Arc` wrapped `Config` instance.
/// * `bot_data` - An `Arc` wrapped `BotData` instance.
///
async fn launch_web_server_thread(
    ctx: Context,
    command_usage: Arc<RwLock<RootUsage>>,
    is_grpc_on: bool,
    config: Arc<Config>,
    bot_data: Arc<BotData>,
) {
    // Check if GRPC is enabled
    if !is_grpc_on {
        info!("API is off, skipping the API server thread!");
        return;
    }

    tokio::spawn(   api::graphql::server::launch(config.bot.config.clone()));

    // Read the data from the context
    let data_read = ctx.data.read().await;

    // Get the ShardManager from the data
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return;
        }
    };

    // Clone the cache and http instances
    let cache = ctx.cache.clone();
    let http = ctx.http.clone();

    info!("GRPC is on, launching the GRPC server thread!");

    // Launch the GRPC server
    grpc_server_launcher(
        shard_manager,
        command_usage,
        cache,
        http,
        config.clone(),
        bot_data.clone(),
    )
    .await
}

/// Asynchronously launches the user color management thread.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance used to access the bot's data and cache.
/// * `db_type` - A `String` representing the type of database being used.
/// * `user_blacklist_server_image` - An `Arc` wrapped `RwLock` containing a list of strings for user blacklist and server image management.
///
async fn launch_user_color_management_thread(
    ctx: Context,
    user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
    db_config: BotConfigDetails,
) {
    // Create an interval for periodic updates
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_USER_COLOR_UPDATE));

    // Log a message indicating that the user color management thread is being launched
    info!("Launching the user color management thread!");

    // Enter a loop that waits for the next interval tick and triggers color management
    loop {
        // Wait for the next interval tick
        interval.tick().await;

        // Get the guilds from the context cache
        let guilds = ctx.cache.guilds();

        // Perform color management for the guilds
        color_management(
            &guilds,
            &ctx,
            user_blacklist_server_image.clone(),
            db_config.clone(),
        )
        .await;
    }
}

/// Asynchronously launches the game management thread.
///
/// # Arguments
///
/// * `apps` - An `Arc` wrapped `RwLock` containing a `HashMap` of `String` keys and `u128` values.
///
async fn launch_game_management_thread(apps: Arc<RwLock<HashMap<String, u128>>>) {
    // Create an interval for periodic updates
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE));

    // Log a message indicating that the steam management thread is being launched
    info!("Launching the steam management thread!");

    // Loop indefinitely
    loop {
        // Wait for the next interval tick
        interval.tick().await;

        // Get the game with the provided apps and wait for the result
        get_game(apps.clone()).await;
    }
}

/// This function is responsible for launching the activity management thread.
/// It takes a `Context` and a `String` as arguments, and does not return anything.
///
/// The `Context` is used to access the bot's data and cache.
/// The `String` is the type of database being used.
///
/// The function creates an interval for periodic updates and logs a message indicating that the thread is being launched.
/// It then enters a loop that waits for the next interval tick and spawns a new task to manage the bot's activity.
/// The task is cloned from the `Context` and `db_type` arguments.
///
async fn launch_activity_management_thread(
    ctx: Context,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: BotConfigDetails,
) {
    // Create an interval for periodic updates
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_ACTIVITY_CHECK));

    // Log a message indicating that the activity management thread is being launched
    info!("Launching the activity management thread!");

    // Enter a loop that waits for the next interval tick and spawns a new task to manage the bot's activity
    loop {
        // Wait for the next interval tick
        interval.tick().await;

        // Clone the context and db_type arguments
        let ctx = ctx.clone();

        // Spawn a new task to manage the bot's activity
        tokio::spawn(manage_activity(
            ctx,
            anilist_cache.clone(),
            db_config.clone(),
        ));
    }
}

/// This function is responsible for launching the server image management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the server image management function.
///
async fn launch_server_image_management_thread(
    ctx: Context,
    image_config: ImageConfig,
    db_config: BotConfigDetails,
) {
    // Log a message indicating that the server image management thread is being launched
    info!("Launching the server image management thread!");

    // Create an interval for periodic updates
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE));

    // Loop indefinitely
    loop {
        // Wait for the next interval tick
        interval.tick().await;

        // Call the server_image_management function with the provided context, database type, and image configuration
        server_image_management(&ctx, image_config.clone(), db_config.clone()).await;
    }
}

/// Asynchronously updates the user blacklist based on the retrieved data from a URL.
///
/// # Arguments
///
/// * `blacklist_lock` - An `Arc` wrapped `RwLock` containing the blacklist data.
///
async fn update_user_blacklist(blacklist_lock: Arc<RwLock<Vec<String>>>) {
    // Create an interval for periodic updates
    let mut interval =
        tokio::time::interval(Duration::from_secs(TIME_BETWEEN_BLACKLISTED_USER_UPDATE));

    loop {
        // Wait for the interval to tick
        interval.tick().await;

        // Fetch the blacklist data from a URL
        let blacklist_url =
            "https://raw.githubusercontent.com/ValgulNecron/kasuki/dev/blacklist.json";
        let blacklist_response = reqwest::get(blacklist_url)
            .await
            .expect("Failed to get blacklist");

        // Parse the JSON response into a Value type
        let blacklist_json: Value = blacklist_response
            .json()
            .await
            .expect("Failed to parse blacklist");

        // Extract user IDs from the JSON array
        let user_ids: Vec<String> = match blacklist_json["user_id"].as_array() {
            Some(arr) => arr,
            None => {
                error!("Failed to get user_id from blacklist");
                continue;
            }
        }
        .iter()
        .map(|id| match id.as_str() {
            Some(id) => id.to_string(),
            None => {
                error!("Failed to get user_id from blacklist");
                "".to_string()
            }
        })
        .collect();

        // Write the updated blacklist to the shared data structure
        let mut blacklist = blacklist_lock.write().await;
        blacklist.clear();
        blacklist.shrink_to_fit();
        *blacklist = user_ids;
    }
}

/// Asynchronously updates the bot information based on the context and bot data.
///
/// # Arguments
///
/// * `context` - A `Context` instance used to retrieve information.
/// * `bot_data` - An `Arc` reference to the `BotData` struct.
///
async fn update_bot_info(context: Context, bot_data: Arc<BotData>) {
    // Create a time interval for updating bot info
    let mut update_interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_BOT_INFO));

    loop {
        // Wait for the update interval
        update_interval.tick().await;

        // Retrieve the current bot information
        let current_bot_info = match context.http.get_current_application_info().await {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to get bot info: {:?}", e);
                continue;
            }
        };

        // Acquire a lock on bot info and update it with the current information
        let mut bot_info_lock = bot_data.bot_info.write().await;
        *bot_info_lock = Some(current_bot_info);
    }
}
