use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::command::user::avatar::get_user_command;
use crate::config::DbConfig;
use crate::event_handler::{BotData, RootUsage};
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::user::command_usage::load_localization_command_usage;
use anyhow::Result;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
    CreateInteractionResponseMessage, User,
};
use tokio::sync::{RwLock, RwLockReadGuard};

pub struct CommandUsageCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for CommandUsageCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for CommandUsageCommand {
    async fn run_slash(&self) -> Result<()> {
        let user = get_user_command(&self.ctx, &self.command_interaction).await?;
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        send_embed(
            &self.ctx,
            &self.command_interaction,
            user,
            &bot_data.config.db.clone(),
            &bot_data.number_of_command_use_per_command,
        )
        .await
    }
}

pub async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    user: User,
    config: &DbConfig,
    command_usage: &Arc<RwLock<RootUsage>>,
) -> Result<()> {
    let db_config = config.clone();

    let user_id = user.id.to_string();

    let username = user.name.clone();

    let read_command_usage = command_usage.read().await;

    let usage = get_usage_for_id(&user_id, read_command_usage);

    let guild_id = command_interaction
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or("0".to_string());

    let localized_command_usage = load_localization_command_usage(guild_id, db_config).await?;

    let embed =
        get_default_embed(None).title(localized_command_usage.title.replace("$user$", &username));

    let mut embeds = Vec::new();

    if usage.is_empty() {
        let inner_embed = embed.description(
            localized_command_usage
                .no_usage
                .replace("$user$", &username),
        );

        embeds.push(inner_embed);
    } else {
        let mut description = String::new();

        let mut inner_embed = embed.clone();

        for (command, usage_count) in &usage {
            description.push_str(
                localized_command_usage
                    .command_usage
                    .replace("$command$", command)
                    .replace("$usage$", &usage_count.to_string())
                    .as_str(),
            );

            description.push('\n');

            if description.len() > 4096 {
                let desc = description.clone();
                embeds.push(inner_embed.clone().description(desc));

                description.clear();

                inner_embed = embed.clone();
            }
        }

        if !description.is_empty() {
            embeds.push(inner_embed.clone().description(description));
        }
    }

    let builder_message = CreateInteractionResponseMessage::new().embeds(embeds);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}

fn get_usage_for_id(
    target_id: &str,
    root_usage: RwLockReadGuard<RootUsage>,
) -> Vec<(String, u128)> {
    let mut usage = Vec::new();

    for (command, user_info) in root_usage.command_list.iter() {
        for (id, user_usage) in user_info.user_info.iter() {
            if id == target_id {
                usage.push((command.clone(), user_usage.usage));
            }
        }
    }

    usage
}
