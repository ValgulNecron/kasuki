use chrono::Utc;
use moka::future::Cache;
use num_bigint::BigUint;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ActivityData, CommandType, Context, CurrentApplicationInfo, EventHandler, Guild, GuildId,
    Interaction, Member, Presence, Ready, User,
};
use serenity::async_trait;
use serenity::gateway::ChunkGuildFilter;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

use crate::autocomplete::autocomplete_dispatch::autocomplete_dispatching;
use crate::background_task::background_launcher::thread_management_launcher;
use crate::background_task::server_image::calculate_user_color::{
    color_management, get_specific_user_color,
};
use crate::background_task::server_image::generate_server_image::server_image_management;
use crate::command::command_dispatch::{check_if_module_is_on, dispatch_command};
use crate::command::user_command_dispatch::dispatch_user_command;
use crate::components::components_dispatch::components_dispatching;
use crate::config::Config;
use crate::constant::COMMAND_USE_PATH;
use crate::get_url;
use crate::helper::error_management::error_dispatch;
use crate::new_member::new_member_message;
use crate::register::registration_dispatcher::command_registration;
use crate::removed_member::removed_member_message;
use crate::structure::database::prelude::{GuildData, UserData};

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
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        let image_config = self.bot_data.config.image.clone();
        let user_blacklist_server_image = self.bot_data.user_blacklist_server_image.clone();
        if is_new.unwrap_or_default() {
            color_management(
                &ctx.cache.guilds(),
                &ctx,
                user_blacklist_server_image,
                self.bot_data.config.db.clone(),
            )
            .await;
            server_image_management(&ctx, image_config, self.bot_data.config.db.clone()).await;
            debug!("Joined a new guild: {} at {}", guild.name, guild.joined_at);
        } else {
            debug!("Got info from guild: {} at {}", guild.name, guild.joined_at);
        }
        let connection =
            match sea_orm::Database::connect(get_url(self.bot_data.config.db.clone())).await {
                Ok(connection) => connection,
                Err(e) => {
                    error!("Failed to connect to the database. {}", e);
                    return;
                }
            };
        match GuildData::insert(crate::structure::database::guild_data::ActiveModel {
            guild_id: Set(guild.id.to_string()),
            guild_name: Set(guild.name),
            updated_at: Set(guild.joined_at.naive_utc()),
        })
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(
                crate::structure::database::guild_data::Column::GuildId,
            )
            .update_column(crate::structure::database::guild_data::Column::GuildName)
            .update_column(crate::structure::database::guild_data::Column::UpdatedAt)
            .to_owned(),
        )
        .exec(&connection)
        .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to insert guild data. {}", e),
        };
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        let user_blacklist_server_image = self.bot_data.user_blacklist_server_image.clone();
        let guild_id = member.guild_id.to_string();
        let image_config = self.bot_data.config.image.clone();
        debug!(
            "Member {} joined guild {}",
            member.user.tag(),
            guild_id.clone()
        );
        let is_module_on = check_if_module_is_on(
            guild_id.clone(),
            "NEW_MEMBER",
            self.bot_data.config.db.clone(),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to get the module status. {}", e);
            false
        });
        match new_member_message(&ctx, &member, self.bot_data.config.db.clone()).await {
            Ok(_) => {}
            Err(e) => error!(e),
        };
        color_management(
            &ctx.cache.guilds(),
            &ctx,
            user_blacklist_server_image,
            self.bot_data.config.db.clone(),
        )
        .await;
        if is_module_on {
            server_image_management(&ctx, image_config, self.bot_data.config.db.clone()).await;
        }
        let connection =
            match sea_orm::Database::connect(get_url(self.bot_data.config.db.clone())).await {
                Ok(connection) => connection,
                Err(e) => {
                    error!("Failed to connect to the database. {}", e);
                    return;
                }
            };
        match UserData::insert(crate::structure::database::user_data::ActiveModel {
            user_id: Set(member.user.id.to_string()),
            username: Set(member.user.name.clone()),
            added_at: Set(Utc::now().naive_utc()),
            is_bot: Set(member.user.bot),
            banner: Set(member.user.banner_url().unwrap_or_default()),
        })
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(
                crate::structure::database::user_data::Column::UserId,
            )
            .update_column(crate::structure::database::user_data::Column::Username)
            .to_owned(),
        )
        .exec(&connection)
        .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to insert user data. {}", e),
        };
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let is_module_on = check_if_module_is_on(
            guild_id.to_string().clone(),
            "NEW_MEMBER",
            self.bot_data.config.db.clone(),
        )
        .await
        .unwrap_or_else(|e| {
            error!("{}", e);
            false
        });
        if is_module_on {
            match removed_member_message(&ctx, guild_id, user, self.bot_data.config.db.clone())
                .await
            {
                Ok(_) => {}
                Err(e) => error!(e),
            }
        }
    }

    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        let user_blacklist_server_image = self.bot_data.user_blacklist_server_image.clone();
        let user_id = new_data.user.id;
        let username = new_data.user.name.unwrap_or_default();
        debug!("Member {} updated presence", user_id);
        let user = match new_data.user.id.to_user(&ctx).await {
            Ok(user) => user,
            Err(e) => {
                error!("Failed to get the user. {}", e);
                return;
            }
        };
        get_specific_user_color(
            user_blacklist_server_image,
            user.clone(),
            self.bot_data.config.db.clone(),
        )
        .await;

        let connection =
            match sea_orm::Database::connect(get_url(self.bot_data.config.db.clone())).await {
                Ok(connection) => connection,
                Err(e) => {
                    error!("Failed to connect to the database. {}", e);
                    return;
                }
            };
        match UserData::insert(crate::structure::database::user_data::ActiveModel {
            user_id: Set(user_id.to_string()),
            username: Set(username),
            added_at: Set(Utc::now().naive_utc()),
            is_bot: Set(user.bot),
            banner: Set(user.banner_url().unwrap_or_default()),
        })
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(
                crate::structure::database::user_data::Column::UserId,
            )
            .update_column(crate::structure::database::user_data::Column::Username)
            .to_owned(),
        )
        .exec(&connection)
        .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to insert user data. {}", e),
        };
    }

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
                self.bot_data.config.db.clone(),
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
        let remove_old_command = self.bot_data.config.bot.remove_old_commands;

        // Creates commands based on the value of the "REMOVE_OLD_COMMAND" environment variable
        command_registration(&ctx.http, remove_old_command).await;
        // Iterates over each guild the bot is in
        let shard = ctx.shard.clone();
        for guild in ctx.cache.guilds() {
            // Retrieves partial guild information
            let partial_guild = match guild.to_partial_guild(&ctx.http).await {
                Ok(guild) => guild,
                Err(e) => {
                    error!("Failed to get the guild. {}", e);
                    continue;
                }
            };
            // Logs the guild name and ID
            shard.chunk_guild(partial_guild.id, None, true, ChunkGuildFilter::None, None);
            debug!(
                "guild name {} (guild id: {})",
                &partial_guild.name,
                &partial_guild.id.to_string()
            )
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let mut user = None;
        if let Interaction::Command(command_interaction) = interaction.clone() {
            let mut message = String::from("");
            if command_interaction.data.kind == CommandType::ChatInput {
                match dispatch_command(&ctx, &command_interaction, self).await {
                    Ok(()) => return,
                    Err(e) => {
                        message = e.to_string();
                    }
                }
            } else if command_interaction.data.kind == CommandType::User {
                match dispatch_user_command(&ctx, &command_interaction, self).await {
                    Ok(()) => return,
                    Err(e) => {
                        message = e.to_string();
                    }
                }
            } else if command_interaction.data.kind == CommandType::Message {
                trace!("{:?}", command_interaction)
            }
            error_dispatch::command_dispatching(message, &command_interaction, &ctx, self).await;
            user = Some(command_interaction.user.clone());
        } else if let Interaction::Autocomplete(autocomplete_interaction) = interaction.clone() {
            // Dispatch the autocomplete interaction
            user = Some(autocomplete_interaction.user.clone());

            autocomplete_dispatching(ctx, autocomplete_interaction, self).await;
        } else if let Interaction::Component(component_interaction) = interaction.clone() {
            // Dispatch the component interaction
            user = Some(component_interaction.user.clone());
            if let Err(e) =
                components_dispatching(ctx, component_interaction, self.bot_data.config.db.clone())
                    .await
            {
                // If an error occurs, log it
                error!("{:?}", e)
            }
        }

        if user.is_none() {
            return;
        }

        let connection =
            match sea_orm::Database::connect(get_url(self.bot_data.config.db.clone())).await {
                Ok(conn) => conn,
                Err(_) => {
                    return;
                }
            };

        match UserData::insert(crate::structure::database::user_data::ActiveModel {
            user_id: Set(user.clone().unwrap().id.to_string()),
            username: Set(user.clone().unwrap().name),
            added_at: Set(Utc::now().naive_utc()),
            is_bot: Set(user.clone().unwrap().bot),
            banner: Set(user.unwrap().banner_url().unwrap_or_default()),
        })
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(
                crate::structure::database::user_data::Column::UserId,
            )
            .update_column(crate::structure::database::user_data::Column::Username)
            .to_owned(),
        )
        .exec(&connection)
        .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to insert user data. {}", e),
        };
    }
}
