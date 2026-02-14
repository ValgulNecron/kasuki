use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::staff::get_staff;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "staff", desc = "Get info of a VN staff member.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the staff member.", arg_type = String, required = true, autocomplete = false)],
)]
async fn vn_staff_command(self_: VnStaffCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let vndb_cache = bot_data.vndb_cache.clone();

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let map = get_option_map_string_subcommand(command_interaction);
	let staff = map
		.get(&String::from("name"))
		.cloned()
		.unwrap_or(String::new());
	let db_connection = bot_data.db_connection.clone();

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let staff = get_staff(staff.clone(), vndb_cache.clone()).await?;

	let staff = staff.results[0].clone();

	let fields = vec![
		(USABLE_LOCALES.lookup(&lang_id, "vn_staff-lang"), staff.lang, true),
		(USABLE_LOCALES.lookup(&lang_id, "vn_staff-aid"), staff.aid.to_string(), true),
		(USABLE_LOCALES.lookup(&lang_id, "vn_staff-gender"), staff.gender.clone().unwrap_or(String::new()), true),
		(USABLE_LOCALES.lookup(&lang_id, "vn_staff-main"), staff.ismain.to_string(), true),
	];
	let staff_desc = staff.description.clone();

	let embed_content = EmbedContent::new(staff.name.clone())
		.description(String::from(convert_vndb_markdown(&staff_desc)))
		.fields(fields)
		.url(format!("https://vndb.org/{}", staff.id));

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}
