
use std::env;
use std::sync::Arc;
use std::time::Duration;



use serenity::all::{
    ActivityData, Context, EventHandler, GatewayIntents, Interaction, Ready, ShardManager,
};
use serenity::all::{Guild, Member};
use serenity::{async_trait, Client};

use tokio::time::sleep;
use tracing::{debug, error, info, trace};

use struct_shard_manager::ShardManagerContainer;

use crate::activity::anime_activity::manage_activity;
use crate::command_autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command_register::command_registration::creates_commands;
use crate::command_run::command_dispatch::{check_if_module_is_on, command_dispatching};
use crate::components::components_dispatch::components_dispatching;
use crate::constant::ACTIVITY_NAME;
use crate::constant::PING_UPDATE_DELAYS;

use crate::constant::{
    TIME_BEFORE_SERVER_IMAGE, TIME_BETWEEN_SERVER_IMAGE_UPDATE, TIME_BETWEEN_USER_COLOR_UPDATE,
};
use crate::constant::{TIME_BETWEEN_GAME_UPDATE};
use crate::database::dispatcher::data_dispatch::set_data_ping_history;
use crate::database::dispatcher::init_dispatch::init_sql_database;
use crate::error_management::error_dispatch;
use crate::game_struct::steam_game_id_struct::get_game;
use crate::logger::{create_log_directory, init_logger};
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
        if check_if_module_is_on(guild_id, "GAME")
            .await
            .unwrap_or(true)
        {
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
/// The main function where the execution of the bot starts.
/// It initializes the logger, the SQL database, and the bot client.
/// It also spawns asynchronous tasks for managing the ping of the shards and starting the client.
async fn main() {
    // Print a message indicating the bot is starting.
    println!("Bot starting please wait.");

    // Load environment variables from the .env file.
    let _ = dotenv::from_path(".env");

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

    // Initialize the SQL database.
    // If an error occurs, log the error and return.
    if let Err(e) = init_sql_database().await {
        error!("{:?}", e);
        return;
    }

    // Log a message indicating the bot is starting.
    info!("starting the bot.");

    // Get the Discord token from the environment variable "DISCORD_TOKEN".
    // If the variable is not set, log an error and exit the process.
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        error!("Env variable not set exiting.");
        std::process::exit(1);
    });
    // Get all the non-privileged intent.
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
    let mut client = Client::builder(token, gateway_intent)
        .event_handler(Handler)
        .await
        .unwrap_or_else(|e| {
            error!("Error while creating client: {}", e);
            std::process::exit(1);
        });

    // Clone the shard manager from the client.
    let shard_manager = client.shard_manager.clone();

    // Insert the cloned shard manager into the client's data.
    // This allows for the shard manager to be accessed from the context in event handlers.
    client
        .data
        .write()
        .await
        .insert::<ShardManagerContainer>(Arc::clone(&shard_manager));

    // Spawn a new asynchronous task for managing the ping of the shards.
    // This task runs indefinitely, pinging the shard manager every `PING_UPDATE_DELAYS` seconds.
    tokio::spawn(async move {
        info!("Launching the ping thread!");
        loop {
            ping_manager(&shard_manager).await;
            sleep(Duration::from_secs(PING_UPDATE_DELAYS)).await;
        }
    });

    // Spawn a new asynchronous task for starting the client.
    // If the client fails to start, log the error.
    tokio::spawn(async move {
        if let Err(why) = client.start_autosharded().await {
            error!("Client error: {:?}", why)
        }
    });

    // Wait for a Ctrl-C signal.
    // If received, print a shutdown message.
    let _signal_err = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}

/// This function is responsible for launching various threads for different tasks.
/// It takes a `Context` as an argument which is used to clone and pass to the threads.
/// The function does not return anything.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used to clone and pass to the threads.
///
async fn thread_management_launcher(ctx: Context) {
    // Get the guilds from the context cache
    // Clone the context
    // Spawn a new thread for the web server
    tokio::spawn(launch_web_server_thread());
    // Spawn a new thread for user color management
    tokio::spawn(launch_user_color_management_thread(ctx.clone()));
    // Spawn a new thread for activity management
    tokio::spawn(launch_activity_management_thread(ctx.clone()));
    // Spawn a new thread for game management
    tokio::spawn(launch_game_management_thread());

    // Sleep for a specified duration before spawning the server image management thread
    sleep(Duration::from_secs(TIME_BEFORE_SERVER_IMAGE)).await;
    let ctx_clone = ctx.clone();
    // Spawn a new thread for server image management
    tokio::spawn(async move {
        info!("Launching the server image management thread!");
        loop {
            server_image_management(&ctx_clone).await;
            sleep(Duration::from_secs(TIME_BETWEEN_SERVER_IMAGE_UPDATE)).await;
        }
    });

    info!("Done spawning thread manager.");
}

/// This function is responsible for launching the web server thread.
/// It does not take any arguments and does not return anything.
async fn launch_web_server_thread() {
    info!("Launching the log web server thread!");
    web_server_launcher().await
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
    info!("Launching the user color management thread!");
    loop {
        let guilds = ctx.cache.guilds();
        color_management(&guilds, &ctx).await;
        sleep(Duration::from_secs(TIME_BETWEEN_USER_COLOR_UPDATE)).await;
    }
}

/// This function is responsible for launching the game management thread.
/// It does not take any arguments and does not return anything.
async fn launch_game_management_thread() {
    info!("Launching the game management thread!");
    loop {
        get_game().await;
        sleep(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE)).await;
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
    info!("Launching the activity management thread!");
    loop {
        manage_activity(&ctx).await;
        sleep(Duration::from_secs(1)).await;
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
