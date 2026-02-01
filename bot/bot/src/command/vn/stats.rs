use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::vndbapi::stats::get_stats;
use crate::impl_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
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
		let vndb_cache = bot_data.vndb_cache.clone();
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
		let lang_id = get_language_identifier(guild_id, db_connection).await;
		debug!("Localization loaded successfully");

		debug!("Creating fields for embed");
		let fields = vec![
			(USABLE_LOCALES.lookup(&lang_id, "vn_stats-chars"), stats.chars.to_string(), true),
			(
				USABLE_LOCALES.lookup(&lang_id, "vn_stats-producer"),
				stats.producers.to_string(),
				true,
			),
			(
				USABLE_LOCALES.lookup(&lang_id, "vn_stats-release"),
				stats.releases.to_string(),
				true,
			),
			(USABLE_LOCALES.lookup(&lang_id, "vn_stats-staff"), stats.staff.to_string(), true),
			(USABLE_LOCALES.lookup(&lang_id, "vn_stats-tags"), stats.tags.to_string(), true),
			(
				USABLE_LOCALES.lookup(&lang_id, "vn_stats-traits"),
				stats.traits.to_string(),
				true,
			),
			(USABLE_LOCALES.lookup(&lang_id, "vn_stats-vns"), stats.vn.to_string(), true),
			(USABLE_LOCALES.lookup(&lang_id, "vn_stats-api"), String::from("VNDB API"), true),
		];
		trace!("Created {} fields for embed", fields.len());

		let title = USABLE_LOCALES.lookup(&lang_id, "vn_stats-title");
		debug!("Creating embed content with title: {}", title);
		let embed_content = EmbedContent::new(title).fields(fields);

		debug!("Creating final embed contents with CommandType::Followup");
		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		info!("VN stats command processed successfully");
		Ok(embed_contents)
	}
);
