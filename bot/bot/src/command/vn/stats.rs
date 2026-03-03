use crate::command::command::CommandRun;
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use shared::vndb::stats::get_stats;
use tracing::{debug, info, trace};

#[slash_command(
	name = "stats", desc = "Get VN statistics.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn vn_stats_command(self_: VnStatsCommand) -> Result<EmbedsContents<'_>> {
	info!("Processing VN stats command");
	debug!("Deferring command response");

	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let vndb_cache = cx.vndb_cache.clone();
	debug!("Retrieved bot data and VNDB cache");

	debug!("Fetching VNDB stats from cache");
	let stats = get_stats(vndb_cache).await?;
	debug!("VNDB stats retrieved successfully");

	debug!("Loading localization for guild: {}", cx.guild_id);
	let lang_id = cx.lang_id().await;
	debug!("Localization loaded successfully");

	debug!("Creating fields for embed");
	let fields = vec![
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-chars"),
			stats.chars.to_string(),
			true,
		),
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
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-staff"),
			stats.staff.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-tags"),
			stats.tags.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-traits"),
			stats.traits.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-vns"),
			stats.vn.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_stats-api"),
			String::from("VNDB API"),
			true,
		),
	];
	trace!("Created {} fields for embed", fields.len());

	let title = USABLE_LOCALES.lookup(&lang_id, "vn_stats-title");
	debug!("Creating embed content with title: {}", title);
	let embed_content = EmbedContent::new(title).fields(fields);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	info!("VN stats command processed successfully");
	Ok(embed_contents)
}
