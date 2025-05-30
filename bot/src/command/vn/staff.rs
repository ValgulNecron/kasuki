use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use crate::structure::message::vn::staff::load_localization_staff;
use anyhow::Result;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};

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

	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let vndb_cache = bot_data.vndb_cache.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let map = get_option_map_string_subcommand(command_interaction);
		let db_config = config.db.clone();
		let staff = map
			.get(&String::from("name"))
			.cloned()
			.unwrap_or(String::new());

		let staff_localised = load_localization_staff(guild_id, db_config).await?;

		let staff = get_staff(staff.clone(), vndb_cache.clone()).await?;

		let staff = staff.results[0].clone();

		let fields = vec![
			(staff_localised.lang.clone(), staff.lang, true),
			(staff_localised.aid.clone(), staff.aid.to_string(), true),
			(staff_localised.gender.clone(), staff.gender.clone(), true),
			(staff_localised.main.clone(), staff.ismain.to_string(), true),
		];
		let staff_desc = staff.description.clone();

		let embed_content = EmbedContent::new(staff.name.clone())
			.description(String::from(convert_vndb_markdown(&staff_desc)))
			.fields(fields)
			.url(format!("https://vndb.org/{}", staff.id));

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}
