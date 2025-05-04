use crate::command::command_trait::{Command, Embed, EmbedContent, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::vndbapi::stats::get_stats;
use crate::structure::message::vn::stats::load_localization_stats;
use anyhow::Result;
use moka::future::Cache;
use serenity::all::{
	CommandInteraction, Context as SerenityContext
	,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct VnStatsCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for VnStatsCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for VnStatsCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let vndb_cache = bot_data.vndb_cache.clone();

		let content = get_content(command_interaction, config, vndb_cache).await?;

		self.send_embed(vec![content]).await
	}
}

async fn get_content(
	command_interaction: &CommandInteraction, config: Arc<Config>,
	vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<EmbedContent<'static, 'static>> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let stats = get_stats(vndb_cache).await?;
	let stats_localised = load_localization_stats(guild_id, config.db.clone()).await?;
	let fields = vec![
		(stats_localised.chars.clone(), stats.chars.to_string(), true),
		(
			stats_localised.producer.clone(),
			stats.producers.to_string(),
			true,
		),
		(
			stats_localised.release.clone(),
			stats.releases.to_string(),
			true,
		),
		(stats_localised.staff.clone(), stats.staff.to_string(), true),
		(stats_localised.staff.clone(), stats.staff.to_string(), true),
		(stats_localised.tags.clone(), stats.tags.to_string(), true),
		(
			stats_localised.traits.clone(),
			stats.traits.to_string(),
			true,
		),
		(stats_localised.vns.clone(), stats.vn.to_string(), true),
		(stats_localised.api.clone(), String::from("VNDB API"), true),
	];

	let content = EmbedContent::new(stats_localised.title).fields(fields);

	Ok(content)
}
