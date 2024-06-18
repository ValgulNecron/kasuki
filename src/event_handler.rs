use std::collections::HashMap;
use std::env;
use std::ops::Add;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serenity::all::{
    ActivityData, CommandType, Context, EventHandler, Guild, Interaction, Member, Ready,
};
use serenity::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

use crate::background_task::background_launcher::thread_management_launcher;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::command::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command::run::command_dispatch::command_dispatching;
use crate::command::user_run::dispatch::dispatch_user_command;
use crate::command_register::registration_dispatcher::command_dispatcher;
use crate::components::components_dispatch::components_dispatching;
use crate::constant::{BOT_INFO, COMMAND_USE_PATH, CONFIG};
use crate::helper::error_management::error_dispatch;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub struct Handler {
    pub number_of_command_use: Arc<RwLock<u128>>,
    pub number_of_command_use_per_command: Arc<RwLock<RootUsage>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUsage {
    pub user_name: String,
    pub usage: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_info: HashMap<String, UserUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootUsage {
    pub command_list: HashMap<String, UserInfo>,
}

impl RootUsage {
    pub fn new() -> Self {
        RootUsage {
            command_list: HashMap::new(),
        }
    }
}

impl Handler {
    // thread safe way to increment the number of command use
    pub async fn increment_command_use(&self) {
        let mut guard = self.number_of_command_use.write().await;
        *guard = guard.add(1);
    }

    // thread safe way to increment the number of command use per command
    pub async fn increment_command_use_per_command(
        &self,
        command_name: String,
        user_id: String,
        user_name: String,
    ) {
        let mut guard = self.number_of_command_use_per_command.write().await;
        let command_map = guard
            .command_list
            .entry(command_name)
            .or_insert_with(|| UserInfo {
                user_info: HashMap::new(),
            });
        let user_map = command_map
            .user_info
            .entry(user_id)
            .or_insert_with(|| UserUsage {
                user_name,
                usage: 0,
            });
        user_map.usage = user_map.usage.add(1);

        // drop the guard
        drop(guard);
        // save the content as a json
        match serde_json::to_string(&*self.number_of_command_use_per_command.read().await) {
            Ok(content) => {
                // save the content to the file
                if let Err(e) = std::fs::write(COMMAND_USE_PATH, content) {
                    error!("Failed to write to file: {}", e);
                }
            }
            Err(e) => error!("Error serializing data: {}", e),
        }
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
    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        let guild_id = member.guild_id.to_string();
        trace!("Member {} joined guild {}", member.user.tag(), guild_id);
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
            BOT_INFO = Some(bot);
        }

        let command_usage = self.number_of_command_use.clone();

        // Spawns a new thread for managing various tasks
        tokio::spawn(thread_management_launcher(ctx.clone(), command_usage));

        // Sets the bot's activity
        ctx.set_activity(Some(ActivityData::custom(unsafe {
            CONFIG.bot.bot_activity.clone()
        })));

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
