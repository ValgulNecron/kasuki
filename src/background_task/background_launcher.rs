use moka::future::Cache;
use serde_json::Value;
use serenity::all::Context;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::info;

use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::background_task::update_random_stats::update_random_stats_launcher;
use crate::config::Config;
use crate::constant::{
    TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_ACTIVITY_CHECK, TIME_BETWEEN_BLACKLISTED_USER_UPDATE,
    TIME_BETWEEN_BOT_INFO, TIME_BETWEEN_GAME_UPDATE, TIME_BETWEEN_PING_UPDATE,
    TIME_BETWEEN_SERVER_IMAGE_UPDATE, TIME_BETWEEN_USER_COLOR_UPDATE,
};
use crate::database::data_struct::ping_history::PingHistory;
use crate::database::manage::dispatcher::data_dispatch::set_data_ping_history;
use crate::event_handler::{BotData, RootUsage};
use crate::grpc_server::launcher::grpc_server_launcher;
use crate::struct_shard_manager::ShardManagerContainer;
use crate::structure::steam_game_id_struct::get_game;

/// This function is responsible for launching various threads for different tasks.
/// It takes a `Context` as an argument which is used to clone and pass to the threads.
/// The function does not return anything.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used to clone and pass to the threads.
///
pub async fn thread_management_launcher(ctx: Context, bot_data: Arc<BotData>) {
    // Get the guilds from the context cache
    // Clone the context
    // Spawn a new thread for the web server
    let command_usage = bot_data.number_of_command_use_per_command.clone();
    let is_grpc_on = bot_data.config.grpc.grpc_is_on;
    let config = bot_data.config.clone();
    let db_type = config.bot.config.db_type.clone();
    let anilist_cache = bot_data.anilist_cache.clone();
    let apps = bot_data.apps.clone();
    let user_blacklist_server_image = bot_data.user_blacklist_server_image.clone();
    // Spawn a new thread for user color management
    tokio::spawn(launch_user_color_management_thread(
        ctx.clone(),
        db_type.clone(),
        user_blacklist_server_image.clone(),
    ));
    // Spawn a new thread for activity management
    tokio::spawn(launch_activity_management_thread(
        ctx.clone(),
        db_type.clone(),
        anilist_cache.clone(),
    ));
    // Spawn a new thread for steam management
    tokio::spawn(launch_game_management_thread(apps));
    // Spawn a new thread for ping management
    tokio::spawn(ping_manager_thread(ctx.clone(), db_type.clone()));
    // Spawn a new thread for updating the user blacklist
    tokio::spawn(update_user_blacklist(user_blacklist_server_image));
    tokio::spawn(update_random_stats_launcher(anilist_cache.clone()));
    tokio::spawn(update_bot_info(ctx.clone(), bot_data.clone()));
    sleep(Duration::from_secs(1)).await;
    tokio::spawn(launch_web_server_thread(
        ctx.clone(),
        command_usage,
        is_grpc_on,
        config,
        bot_data.clone(),
    ));
    // Sleep for a specified duration before spawning the server image management thread
    sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;
    tokio::spawn(launch_server_image_management_thread(
        ctx.clone(),
        db_type.clone(),
    ));

    info!("Done spawning thread manager.");
}

/// This function is responsible for managing the ping of the shards.
async fn ping_manager_thread(ctx: Context, db_type: String) {
    info!("Launching the ping thread!");
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return;
        }
    };
    let mut interval = tokio::time::interval(Duration::from_secs(TIME_BETWEEN_PING_UPDATE));
    loop {
        interval.tick().await;
        // Lock the runners
        let runner = shard_manager.runners.lock().await;
        // Iterate over the runners
        for (shard_id, shard) in runner.iter() {
            // Get the latency of the shard
            let latency = shard.latency.unwrap_or_default().as_millis().to_string();
            // Set the ping history data
            let now = chrono::Utc::now().timestamp().to_string();
            let ping_history = PingHistory {
                shard_id: shard_id.to_string(),
                ping: latency.clone(),
                timestamp: now,
            };

            set_data_ping_history(ping_history, db_type.clone())
                .await
                .unwrap();
        }
    }
}

/// This function is responsible for launching the web server thread.
/// It does not take any arguments and does not return anything.
async fn launch_web_server_thread(
    ctx: Context,
    command_usage: Arc<RwLock<RootUsage>>,
    is_grpc_on: bool,
    config: Arc<Config>,
    bot_data: Arc<BotData>,
) {
    if !is_grpc_on {
        info!("GRPC is off, skipping the GRPC server thread!");
        return;
    }
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return;
        }
    };
    let cache = ctx.cache.clone();
    let http = ctx.http.clone();
    info!("GRPC is on, launching the GRPC server thread!");

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

/// This function is responsible for launching the user color management thread.
/// It takes a vector of `GuildId` and a `Context` as arguments.
///
/// # Arguments
///
/// * `guilds` - A vector of `GuildId` which is used in the color management function.
/// * `ctx` - A `Context` instance which is used in the color management function.
///
async fn launch_user_color_management_thread(
    ctx: Context,
    db_type: String,
    user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
) {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_USER_COLOR_UPDATE));
    info!("Launching the user color management thread!");
    loop {
        interval.tick().await;
        let guilds = ctx.cache.guilds();
        color_management(
            &guilds,
            &ctx,
            db_type.clone(),
            user_blacklist_server_image.clone(),
        )
        .await;
    }
}

/// This function is responsible for launching the steam management thread.
/// It does not take any arguments and does not return anything.
async fn launch_game_management_thread(apps: Arc<RwLock<HashMap<String, u128>>>) {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE));
    info!("Launching the steam management thread!");
    loop {
        interval.tick().await;
        get_game(apps.clone()).await;
    }
}

/// This function is responsible for launching the activity management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the manage activity function.
///
async fn launch_activity_management_thread(
    ctx: Context,
    db_type: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_ACTIVITY_CHECK));
    info!("Launching the activity management thread!");
    loop {
        interval.tick().await;
        let ctx = ctx.clone();
        let db_type = db_type.clone();
        tokio::spawn(manage_activity(ctx, db_type, anilist_cache.clone()));
    }
}

/// This function is responsible for launching the server image management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the server image management function.
///
async fn launch_server_image_management_thread(ctx: Context, db_type: String) {
    info!("Launching the server image management thread!");
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE));
    loop {
        interval.tick().await;
        server_image_management(&ctx, db_type.clone()).await;
    }
}

async fn update_user_blacklist(user_blacklist_server_image: Arc<RwLock<Vec<String>>>) {
    info!("Launching the user blacklist update thread!");
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_BLACKLISTED_USER_UPDATE));
    loop {
        interval.tick().await;
        // Get a write lock on USER_BLACKLIST_SERVER_IMAGE

        // Perform operations on the data while holding the lock
        let file_url = "https://raw.githubusercontent.com/ValgulNecron/kasuki/dev/blacklist.json";
        let blacklist = reqwest::get(file_url)
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let user_ids: Vec<String> = blacklist["user_id"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();

        let mut user_blacklist = user_blacklist_server_image.write().await;

        // liberate the memory used by the old user_blacklist
        user_blacklist.clear();
        user_blacklist.shrink_to_fit();
        // Update the USER_BLACKLIST_SERVER_IMAGE
        *user_blacklist = user_ids;
        user_blacklist.shrink_to_fit();
        // Release the lock before sleeping
        drop(user_blacklist);
    }
}

async fn update_bot_info(ctx: Context, bot_data: Arc<BotData>) {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_BOT_INFO));
    loop {
        interval.tick().await;
        let bot = ctx.http.get_current_application_info().await.unwrap();
        let mut guard = bot_data.bot_info.write().await;
        *guard = Some(bot);
    }
}
