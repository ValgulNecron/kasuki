use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use crate::structure::message::vn::staff::load_localization_staff;
use anyhow::{Context, Result};
use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

pub struct VnStaffCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for VnStaffCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for VnStaffCommand {
    async fn run_slash(&self) -> Result<()> {
        send_embed(&self.ctx, &self.command_interaction).await
    }
}

async fn send_embed(ctx: &SerenityContext, command_interaction: &CommandInteraction) -> Result<()> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let map = get_option_map_string_subcommand(command_interaction);

    trace!("{:?}", map);
    let bot_data = ctx.data::<BotData>().clone();

    let staff = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    let staff_localised = load_localization_staff(guild_id, bot_data.config.db.clone()).await?;

    let staff = get_staff(staff.clone(), bot_data.vndb_cache.clone()).await?;

    let staff = staff.results[0].clone();

    let fields = vec![
        (staff_localised.lang.clone(), staff.lang, true),
        (staff_localised.aid.clone(), staff.aid.to_string(), true),
        (staff_localised.gender.clone(), staff.gender.clone(), true),
        (staff_localised.main.clone(), staff.ismain.to_string(), true),
    ];
    let staff_desc = staff.description.clone();
    let builder_embed = get_default_embed(None)
        .description(convert_vndb_markdown(&staff_desc))
        .fields(fields)
        .title(staff.name.clone())
        .url(format!("https://vndb.org/{}", staff.id));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await?;

    Ok(())
}
