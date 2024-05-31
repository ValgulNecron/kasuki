use std::sync::Arc;
use std::time::Duration;
use serde_json::Value;
use serenity::all::{Context, ShardManager};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::info;
use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::constant::{GRPC_IS_ON, PING_UPDATE_DELAYS, TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_GAME_UPDATE, TIME_BETWEEN_SERVER_IMAGE_UPDATE, TIME_BETWEEN_USER_COLOR_UPDATE, USER_BLACKLIST_SERVER_IMAGE};
use crate::database::manage::dispatcher::data_dispatch::set_data_ping_history;
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
pub async fn thread_management_launcher(ctx: Context, command_usage: Arc<RwLock<u128>>) {
    // Get the guilds from the context cache
    // Clone the context
    // Spawn a new thread for the web server

    tokio::spawn(launch_web_server_thread(ctx.clone(), command_usage));
    // Spawn a new thread for user color management
    tokio::spawn(launch_user_color_management_thread(ctx.clone()));
    // Spawn a new thread for activity management
    tokio::spawn(launch_activity_management_thread(ctx.clone()));
    // Spawn a new thread for steam management
    tokio::spawn(launch_game_management_thread());
    // Spawn a new thread for ping management
    tokio::spawn(ping_manager_thread(ctx.clone()));
    // Spawn a new thread for updating the user blacklist
    unsafe {
        let local_user_blacklist = USER_BLACKLIST_SERVER_IMAGE.clone();
        tokio::spawn(update_user_blacklist(local_user_blacklist));
    }

    // Sleep for a specified duration before spawning the server image management thread
    sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;
    tokio::spawn(launch_server_image_management_thread(ctx.clone()));

    info!("Done spawning thread manager.");
}

/// This function is responsible for managing the ping of the shards.
async fn ping_manager_thread(ctx: Context) {
    info!("Launching the ping thread!");
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => {
            return;
        }
    };
    let mut interval = tokio::time::interval(Duration::from_secs(PING_UPDATE_DELAYS));
    loop {
        interval.tick().await;
        ping_manager(shard_manager).await;
    }
}

/// This function is responsible for launching the web server thread.
/// It does not take any arguments and does not return anything.
async fn launch_web_server_thread(ctx: Context, command_usage: Arc<RwLock<u128>>) {
    let is_grpc_on = *GRPC_IS_ON;
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
    grpc_server_launcher(shard_manager, command_usage, cache, http).await
}

/// This function is responsible for launching the user color management thread.
/// It takes a vector of `GuildId` and a `Context` as arguments.
///
/// # Arguments
///
/// * `guilds` - A vector of `GuildId` which is used in the color management function.
/// * `ctx` - A `Context` instance which is used in the color management function.
///
async fn launch_user_color_management_thread(ctx: Context) {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_USER_COLOR_UPDATE));
    info!("Launching the user color management thread!");
    loop {
        interval.tick().await;
        let guilds = ctx.cache.guilds();
        color_management(&guilds, &ctx).await;
    }
}

/// This function is responsible for launching the steam management thread.
/// It does not take any arguments and does not return anything.
async fn launch_game_management_thread() {
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE));
    info!("Launching the steam management thread!");
    loop {
        interval.tick().await;
        get_game().await;
    }
}

/// This function is responsible for launching the activity management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the manage activity function.
///
async fn launch_activity_management_thread(ctx: Context) {
    let mut interval = interval(Duration::from_secs(1));
    info!("Launching the activity management thread!");
    loop {
        interval.tick().await;
        tokio::spawn(manage_activity(ctx.clone()));
    }
}

/// This function is responsible for managing the ping of the shards.
/// It takes a reference to an `Arc<ShardManager>` as an argument.
///
/// # Arguments
///
/// * `shard_manager` - A reference to an `Arc<ShardManager>` which is used to get the runners.
///
async fn ping_manager(shard_manager: &Arc<ShardManager>) {
    // Lock the runners
    let runner = shard_manager.runners.lock().await;
    // Iterate over the runners
    for (shard_id, shard) in runner.iter() {
        // Get the latency of the shard
        let latency = shard.latency.unwrap_or_default().as_millis().to_string();
        // Set the ping history data
        set_data_ping_history(shard_id.to_string(), latency)
            .await
            .unwrap();
    }
}

/// This function is responsible for launching the server image management thread.
/// It takes a `Context` as an argument.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used in the server image management function.
///
async fn launch_server_image_management_thread(ctx: Context) {
    info!("Launching the server image management thread!");
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE));
    loop {
        interval.tick().await;
        server_image_management(&ctx).await;
    }
}

async fn update_user_blacklist(user_blacklist_server_image: Arc<RwLock<Vec<String>>>) {
    info!("Launching the user blacklist update thread!");
    let mut interval = interval(Duration::from_secs(3600));
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
