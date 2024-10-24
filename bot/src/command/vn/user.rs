use anyhow::{Context, Error, Result};
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::error_management::error_dispatch;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::user::get_user;
use crate::structure::message::vn::user::load_localization_user;
use crate::structure::message::vn::user::UserLocalised;
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

pub struct VnUserCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub vndb_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for VnUserCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for VnUserCommand {
    async fn run_slash(&self) -> Result<()> {
        send_embed(
            &self.ctx,
            &self.command_interaction,
            self.config.clone(),
            self.vndb_cache.clone(),
        )
        .await
    }
}

async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let map = get_option_map_string_subcommand(command_interaction);

    let user = map
        .get(&String::from("username"))
        .ok_or(Error::from("No username provided"))?;

    let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);

    let user = get_user(path, vndb_cache).await?;

    let user_localised: UserLocalised = load_localization_user(guild_id, config.db.clone()).await?;

    let fields = vec![
        (user_localised.id.clone(), user.id.clone(), true),
        (
            user_localised.playtime.clone(),
            user.lengthvotes.to_string(),
            true,
        ),
        (
            user_localised.playtimesum.clone(),
            user.lengthvotes_sum.to_string(),
            true,
        ),
        (user_localised.name.clone(), user.username.clone(), true),
    ];

    let builder_embed = get_default_embed(None)
        .title(user_localised.title)
        .fields(fields);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
