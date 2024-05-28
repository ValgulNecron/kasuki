use std::collections::HashMap;
use std::env;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use serenity::all::{
    ActivityData, CommandType, Context, EventHandler, GatewayIntents, Interaction, Ready,
    ShardManager,
};
use serenity::all::{Guild, Member};
use serenity::{async_trait, Client};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, trace};

use struct_shard_manager::ShardManagerContainer;

use crate::background_task::activity::anime_activity::manage_activity;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::command::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command::run::command_dispatch::{check_if_module_is_on, command_dispatching};
use crate::command::user_run::dispatch::dispatch_user_command;
use crate::command_register::registration_dispatcher::command_dispatcher;
use crate::components::components_dispatch::components_dispatching;
use crate::constant::{ACTIVITY_NAME, USER_BLACKLIST_SERVER_IMAGE};
use crate::constant::{
    APP_TUI, BOT_INFO, DISCORD_TOKEN, GRPC_IS_ON, TIME_BEFORE_SERVER_IMAGE,
    TIME_BETWEEN_USER_COLOR_UPDATE,
};
use crate::constant::{PING_UPDATE_DELAYS, TIME_BETWEEN_GAME_UPDATE};
use crate::database::manage::dispatcher::data_dispatch::set_data_ping_history;
use crate::database::manage::dispatcher::init_dispatch::init_sql_database;
use crate::grpc_server::launcher::grpc_server_launcher;
use crate::helper::error_management::error_dispatch;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::logger::{create_log_directory, init_logger};
use crate::new_member::new_member;
use crate::structure::steam_game_id_struct::get_game;

mod api;
mod background_task;
mod cache;
mod command;
mod command_register;
mod components;
pub(crate) mod constant;
mod database;
mod grpc_server;
mod helper;
mod logger;
mod new_member;
mod struct_shard_manager;
mod structure;
mod tui;

struct Handler {
    pub number_of_command_use: Arc<RwLock<u128>>,
    pub number_of_command_use_per_command: Arc<RwLock<HashMap<String, HashMap<String, u128>>>>,
}

impl Handler {
    // thread safe way to increment the number of command use
    pub async fn increment_command_use(&self) {
        let mut guard = self.number_of_command_use.write().await;
        *guard = guard.add(1);
    }

    // thread safe way to increment the number of command use per command
    pub async fn increment_command_use_per_command(&self, command_name: String, user_id: String) {
        let mut guard = self.number_of_command_use_per_command.write().await;
        let command_name = command_name.to_string();
        let user_id = user_id.to_string();
        let command_use = guard.entry(command_name).or_insert(HashMap::new());
        let user_use = command_use.entry(user_id).or_insert(0);
        *user_use = user_use.add(1);

        // drop the guard
        drop(guard);
        // save the content as a json
        let content =
            serde_json::to_string(&*self.number_of_command_use_per_command.read().await).unwrap();
        // save the content to the file
        std::fs::write("command_use.json", content).unwrap();
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// This function is called when the bot joins a new guild or when it receives information about a guild it is already a part of.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `Context` instance which provides data and functionality for the command.
    /// * `guild` - The `Guild` that was created or updated.
    /// * `is_new` - A boolean indicating whether the guild is new (i.e., the bot has just joined it).
    ///
    /// # Behavior
    ///
    /// If the bot has just joined a new guild, it performs color management, generates a server image, and logs a debug message.
    /// If the bot has received information about a guild it is already a part of, it simply logs a debug message.
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.unwrap_or_default() {
            color_management(&ctx.cache.guilds(), &ctx).await;
            server_image_management(&ctx).await;
            debug!("Joined a new guild: {} at {}", guild.name, guild.joined_at);
        } else {
            debug!("Got info from guild: {} at {}", guild.name, guild.joined_at);
        }
    }

    /// This function is called when a new member joins a guild.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `Context` instance which provides data and functionality for the command.
    /// * `member` - The `Member` that joined the guild.
    ///
    /// # Behavior
    ///
    /// The function performs color management, generates a server image, and logs a trace message.
    /// If the "NEW_MEMBER" module is on, it calls the `new_member` function to handle the new member.
    /// If an error occurs during the handling of the new member, it logs the error.
    async fn guild_member_addition(&self, ctx: Context, mut member: Member) {
        let guild_id = member.guild_id.to_string();
        trace!("Member {} joined guild {}", member.user.tag(), guild_id);
        if check_if_module_is_on(guild_id, "NEW_MEMBER")
            .await
            .unwrap_or(true)
        {
            let ctx2 = ctx.clone();
            if let Err(e) = new_member(ctx2, &mut member).await {
                error!("{:?}", e)
            }
        }
        color_management(&ctx.cache.guilds(), &ctx).await;
        server_image_management(&ctx).await;
    }

    /// This function is called when the bot is ready.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `Context` instance which provides data and functionality for the command.
    /// * `ready` - The `Ready` event that was triggered.
    ///
    /// # Behavior
    ///
    /// The function performs the following actions:
    /// 1. Spawns a new thread for managing various tasks.
    /// 2. Sets the bot's activity.
    /// 3. Logs a message indicating that the shard is connected.
    /// 4. Logs the number of servers the bot is in.
    /// 5. Loads environment variables from the .env file.
    /// 6. Checks if the "REMOVE_OLD_COMMAND" environment variable is set to "true" (case-insensitive).
    /// 7. Logs the value of the "REMOVE_OLD_COMMAND" environment variable.
    /// 8. Creates commands based on the value of the "REMOVE_OLD_COMMAND" environment variable.
    /// 9. Iterates over each guild the bot is in, retrieves partial guild information, and logs the guild name and ID.
    async fn ready(&self, ctx: Context, ready: Ready) {
        let bot = ctx.http.get_current_application_info().await.unwrap();
        unsafe {
            *BOT_INFO = Some(bot);
        }

        let command_usage = self.number_of_command_use.clone();

        // Spawns a new thread for managing various tasks
        tokio::spawn(thread_management_launcher(ctx.clone(), command_usage));

        // Sets the bot's activity
        ctx.set_activity(Some(ActivityData::custom(ACTIVITY_NAME.clone())));

        // Logs a message indicating that the shard is connected
        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        // Logs the number of servers the bot is in
        let server_number = ctx.cache.guilds().len();
        info!(server_number);

        // Loads environment variables from the .env file
        dotenvy::from_path(".env").ok();

        // Checks if the "REMOVE_OLD_COMMAND" environment variable is set to "true" (case-insensitive)
        let remove_old_command = env::var("REMOVE_OLD_COMMAND")
            .unwrap_or_else(|_| "false".to_string())
            .eq_ignore_ascii_case("true");

        // Logs the value of the "REMOVE_OLD_COMMAND" environment variable
        trace!(remove_old_command);

        // Creates commands based on the value of the "REMOVE_OLD_COMMAND" environment variable
        command_dispatcher(&ctx.http, remove_old_command).await;
        // Iterates over each guild the bot is in
        for guild in ctx.cache.guilds() {
            // Retrieves partial guild information
            let partial_guild = guild.to_partial_guild(&ctx.http).await.unwrap();
            // Logs the guild name and ID
            debug!(
                "guild name {} (guild id: {})",
                &partial_guild.name,
                &partial_guild.id.to_string()
            )
        }
    }

    /// Handles different types of interactions.
    ///
    /// This function is called when an interaction is created. It checks the type of the interaction
    /// and performs the appropriate action based on the type.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `Context` instance which provides data and functionality for the command.
    /// * `interaction` - The `Interaction` that was created.
    ///
    /// # Behavior
    ///
    /// * `Interaction::Command` - Logs the command details and dispatches the command.
    /// If an error occurs during dispatching, it is handled by `error_dispatch::command_dispatching`.
    /// * `Interaction::Autocomplete` - Dispatches the autocomplete interaction.
    /// * `Interaction::Component` - Dispatches the component interaction. If an error occurs, it is logged.
    ///
    /// Other types of interactions are ignored.
    ///
    /// # Errors
    ///
    /// This function does not return any errors. However, it logs errors that occur during the dispatching of commands and components.
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command_interaction) = interaction.clone() {
            // call self.increment_command_use() in a way that would not block the event loop

            if command_interaction.data.kind == CommandType::ChatInput {
                // Log the details of the command interaction
                info!(
                    "Received {} from {} in {} with option {:?}",
                    command_interaction.data.name,
                    command_interaction.user.name,
                    command_interaction.guild_id.unwrap_or_default().to_string(),
                    command_interaction.data.options
                );
                if let Err(e) = command_dispatching(&ctx, &command_interaction, self).await {
                    error_dispatch::command_dispatching(e, &command_interaction, &ctx).await
                }
            } else if command_interaction.data.kind == CommandType::User {
                if let Err(e) = dispatch_user_command(&ctx, &command_interaction).await {
                    error_dispatch::command_dispatching(e, &command_interaction, &ctx).await
                }
            } else if command_interaction.data.kind == CommandType::Message {
                trace!("{:?}", command_interaction)
            } else {
                let e = AppError {
                    message: "Command kind invalid".to_string(),
                    error_type: ErrorType::Command,
                    error_response_type: ErrorResponseType::Message,
                };
                error_dispatch::command_dispatching(e, &command_interaction, &ctx).await
            }
            self.increment_command_use().await;
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            // Dispatch the autocomplete interaction
            autocomplete_dispatching(ctx, autocomplete_interaction).await
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            // Dispatch the component interaction
            if let Err(e) = components_dispatching(ctx, component_interaction).await {
                // If an error occurs, log it
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

    if *APP_TUI {
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

    // Log a message indicating the bot is starting.
    info!("starting the bot.");

    let number_of_command_use = Arc::new(RwLock::new(0u128));
    let number_of_command_use_per_command: HashMap<String, HashMap<String, u128>>;
    // populate the number_of_command_use_per_command with the content of the file
    if let Ok(content) = std::fs::read_to_string("command_use.json") {
        number_of_command_use_per_command = serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
    } else {
        number_of_command_use_per_command = HashMap::new();
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
    let mut client = Client::builder(DISCORD_TOKEN.to_string(), gateway_intent)
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

/// This function is responsible for launching various threads for different tasks.
/// It takes a `Context` as an argument which is used to clone and pass to the threads.
/// The function does not return anything.
///
/// # Arguments
///
/// * `ctx` - A `Context` instance which is used to clone and pass to the threads.
///
async fn thread_management_launcher(ctx: Context, command_usage: Arc<RwLock<u128>>) {
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
    let mut interval = interval(Duration::from_secs(3600));
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
