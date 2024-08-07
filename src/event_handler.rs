use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::Arc;

use moka::future::Cache;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ActivityData, CommandType, Context, CurrentApplicationInfo, EventHandler, Guild, GuildId,
    Interaction, Member, Ready, User,
};
use serenity::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

use crate::background_task::background_launcher::thread_management_launcher;
use crate::background_task::server_image::calculate_user_color::color_management;
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::command::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::command::run::command_dispatch::{check_if_module_is_on, command_dispatching};
use crate::command::user_run::dispatch::dispatch_user_command;
use crate::command_register::registration_dispatcher::command_registration;
use crate::components::components_dispatch::components_dispatching;
use crate::config::Config;
use crate::constant::COMMAND_USE_PATH;
use crate::helper::error_management::error_dispatch;
use crate::helper::error_management::error_enum::{
    FollowupError, ResponseError, UnknownResponseError,
};
use crate::new_member::new_member_message;
use crate::removed_member::removed_member_message;

pub struct BotData {
    pub number_of_command_use_per_command: Arc<RwLock<RootUsage>>,
    pub config: Arc<Config>,
    pub bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
    pub vndb_cache: Arc<RwLock<Cache<String, String>>>,
    pub already_launched: RwLock<bool>,
    pub apps: Arc<RwLock<HashMap<String, u128>>>,
    pub user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
}

pub struct Handler {
    pub bot_data: Arc<BotData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserUsage {
    pub user_name: String,
    pub usage: u128,
    pub hourly_usage: HashMap<String, u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserInfo {
    pub user_info: HashMap<String, UserUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RootUsage {
    pub command_list: HashMap<String, UserInfo>,
}

impl RootUsage {
    pub fn new() -> Self {
        RootUsage {
            command_list: HashMap::new(),
        }
    }
    pub fn get_total_command_use(&self) -> String {
        let mut total = BigUint::ZERO;
        let command_usage = self.clone();
        for (_, user_info) in command_usage.command_list.iter() {
            for (_, user_usage) in user_info.user_info.iter() {
                total.add_assign(user_usage.usage)
            }
        }

        total.to_string()
    }
}

impl Handler {
    pub async fn get_hourly_usage(&self, command_name: String, user_id: String) -> u128 {
        let bot_data = self.bot_data.clone();
        let number_of_command_use_per_command = bot_data.number_of_command_use_per_command.clone();
        let guard = number_of_command_use_per_command.read().await;
        let user_map = guard
            .command_list
            .get(&command_name)
            .cloned()
            .unwrap_or_default()
            .user_info
            .get(&user_id)
            .cloned()
            .unwrap_or_default();
        *user_map
            .hourly_usage
            .get(&chrono::Local::now().format("%H").to_string())
            .unwrap_or(&(0u128))
    }
    // thread safe way to increment the number of command use per command
    pub async fn increment_command_use_per_command(
        &self,
        command_name: String,
        user_id: String,
        user_name: String,
    ) {
        let bot_data = self.bot_data.clone();
        let number_of_command_use_per_command = bot_data.number_of_command_use_per_command.clone();
        let mut guard = number_of_command_use_per_command.write().await;
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
                hourly_usage: Default::default(),
            });
        user_map.usage = user_map.usage.add(1);
        // create a timestamp in format dd:mm:aaaa:hh
        let timestamp = chrono::Local::now().format("%d:%m:%Y:%H").to_string();
        // insert or update the hourly usage
        let hourly_usage = user_map.hourly_usage.entry(timestamp).or_insert(0);
        *hourly_usage += 1;

        // drop the guard
        drop(guard);
        // save the content as a json
        match serde_json::to_string(&*self.bot_data.number_of_command_use_per_command.read().await)
        {
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
        let db_type = self.bot_data.config.bot.config.db_type.clone();
        let image_config = self.bot_data.config.image.clone();
        let user_blacklist_server_image = self.bot_data.user_blacklist_server_image.clone();
        if is_new.unwrap_or_default() {
            color_management(
                &ctx.cache.guilds(),
                &ctx,
                db_type.clone(),
                user_blacklist_server_image,
            )
            .await;
            server_image_management(&ctx, db_type, image_config).await;
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
        let user_blacklist_server_image = self.bot_data.user_blacklist_server_image.clone();
        let db_type = self.bot_data.config.bot.config.db_type.clone();
        let guild_id = member.guild_id.to_string();
        let image_config = self.bot_data.config.image.clone();
        debug!(
            "Member {} joined guild {}",
            member.user.tag(),
            guild_id.clone()
        );
        let is_module_on = check_if_module_is_on(guild_id.clone(), "NEW_MEMBER", db_type.clone())
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get the module status. {}", e);
                false
            });
        new_member_message(&ctx, &member).await;
        color_management(
            &ctx.cache.guilds(),
            &ctx,
            db_type.clone(),
            user_blacklist_server_image,
        )
        .await;
        if is_module_on {
            server_image_management(&ctx, db_type.clone(), image_config).await;
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let db_type = self.bot_data.config.bot.config.db_type.clone();
        let is_module_on =
            check_if_module_is_on(guild_id.to_string().clone(), "NEW_MEMBER", db_type.clone())
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to get the module status. {}", e);
                    false
                });
        if is_module_on {
            removed_member_message(&ctx, guild_id, user).await
        }
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
        // Spawns a new thread for managing various tasks
        let guard = self.bot_data.already_launched.read().await;
        if !(*guard) {
            drop(guard);
            let mut write_guard = self.bot_data.already_launched.write().await;
            *write_guard = true;
            tokio::spawn(thread_management_launcher(
                ctx.clone(),
                self.bot_data.clone(),
            ));
            drop(write_guard)
        }
        // Sets the bot's activity
        ctx.set_activity(Some(ActivityData::custom(
            self.bot_data.config.bot.bot_activity.clone(),
        )));

        // Logs a message indicating that the shard is connected
        info!(
            "Shard {:?} of {} is connected!",
            ready.shard, ready.user.name
        );

        // Logs the number of servers the bot is in
        let server_number = ctx.cache.guilds().len();
        info!(server_number);

        // Checks if the "REMOVE_OLD_COMMAND" environment variable is set to "true" (case-insensitive)
        let remove_old_command = self.bot_data.config.bot.config.remove_old_commands;

        // Creates commands based on the value of the "REMOVE_OLD_COMMAND" environment variable
        command_registration(&ctx.http, remove_old_command).await;
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
            if command_interaction.data.kind == CommandType::ChatInput {
                // Log the details of the command interaction
                info!(
                    "Received {} from {} in {} with option {:?}",
                    command_interaction.data.name,
                    command_interaction.user.name,
                    command_interaction.guild_id.unwrap_or_default().to_string(),
                    command_interaction.data.options
                );
                let message;
                let error_type;
                match command_dispatching(&ctx, &command_interaction, self).await {
                    Ok(()) => return,
                    Err(e) => {
                        message = e.to_string();
                        error_type = if e.is::<ResponseError>() {
                            "ResponseError"
                        } else if e.is::<FollowupError>() {
                            "FollowupError"
                        } else if e.is::<UnknownResponseError>() {
                            "UnknownResponseError"
                        } else {
                            "other"
                        };
                    }
                }
                error_dispatch::command_dispatching(
                    message,
                    error_type,
                    &command_interaction,
                    &ctx,
                    self,
                )
                .await
            } else if command_interaction.data.kind == CommandType::User {
                let message;
                let error_type;
                match dispatch_user_command(&ctx, &command_interaction, self).await {
                    Ok(()) => return,
                    Err(e) => {
                        message = e.to_string();
                        error_type = if e.is::<ResponseError>() {
                            "ResponseError"
                        } else if e.is::<FollowupError>() {
                            "FollowupError"
                        } else if e.is::<UnknownResponseError>() {
                            "UnknownResponseError"
                        } else {
                            "other"
                        };
                    }
                }
                error_dispatch::command_dispatching(
                    message,
                    error_type,
                    &command_interaction,
                    &ctx,
                    self,
                )
                .await
            } else if command_interaction.data.kind == CommandType::Message {
                trace!("{:?}", command_interaction)
            }
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            let db_type = self.bot_data.config.bot.config.db_type.clone();
            let anilist_cache = self.bot_data.anilist_cache.clone();
            let vndb_cache = self.bot_data.vndb_cache.clone();
            let apps = self.bot_data.apps.clone();
            // Dispatch the autocomplete interaction
            autocomplete_dispatching(
                ctx,
                autocomplete_interaction,
                anilist_cache,
                db_type,
                vndb_cache,
                apps,
            )
            .await
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            let db_type = self.bot_data.config.bot.config.db_type.clone();
            let anilist_cache = self.bot_data.anilist_cache.clone();
            // Dispatch the component interaction
            if let Err(e) =
                components_dispatching(ctx, component_interaction, db_type, anilist_cache).await
            {
                // If an error occurs, log it
                error!("{:?}", e)
            }
        }
    }
}
