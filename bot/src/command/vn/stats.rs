use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::vndbapi::stats::get_stats;
use crate::impl_command;
use crate::structure::message::vn::stats::load_localization_stats;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tracing::{debug, info, trace};

#[derive(Clone)]
pub struct VnStatsCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for VnStatsCommand,
	get_contents = |self_: VnStatsCommand| async move {
		info!("Processing VN stats command");
		debug!("Deferring command response");
		self_.defer().await?;

		let ctx = self_.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction();
		let vndb_cache = bot_data.vndb_cache.read().await.get_cache();
		debug!("Retrieved bot data and VNDB cache");

		let guild_id = match command_interaction.guild_id {
			Some(id) => {
				debug!("Command executed in guild: {}", id);
				id.to_string()
			},
			None => {
				debug!("Command executed in DM");
				String::from("0")
			},
		};
		let db_connection = bot_data.db_connection.clone();

		debug!("Fetching VNDB stats from cache");
		let stats = get_stats(vndb_cache).await?;
		debug!("VNDB stats retrieved successfully");

		debug!("Loading localization for guild: {}", guild_id);
		let stats_localised = load_localization_stats(guild_id, db_connection).await?;
		debug!("Localization loaded successfully");

		debug!("Creating fields for embed");
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
			(stats_localised.tags.clone(), stats.tags.to_string(), true),
			(
				stats_localised.traits.clone(),
				stats.traits.to_string(),
				true,
			),
			(stats_localised.vns.clone(), stats.vn.to_string(), true),
			(stats_localised.api.clone(), String::from("VNDB API"), true),
		];
		trace!("Created {} fields for embed", fields.len());

		debug!("Creating embed content with title: {}", stats_localised.title);
		let embed_content = EmbedContent::new(stats_localised.title).fields(fields);

		debug!("Creating final embed contents with CommandType::Followup");
		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		info!("VN stats command processed successfully");
		Ok(embed_contents)
	}
);
