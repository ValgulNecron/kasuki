use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::vndbapi::stats::get_stats;
use crate::structure::message::vn::stats::load_localization_stats;
use serenity::all::{CommandInteraction, Context as SerenityContext};

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

	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let vndb_cache = bot_data.vndb_cache.clone();

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

		let embed_content = EmbedContent::new(stats_localised.title).fields(fields);

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}
