use crate::command::command::CommandRun;
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use shared::vndb::staff::get_staff;

#[slash_command(
	name = "staff", desc = "Get info of a VN staff member.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the staff member.", arg_type = String, required = true, autocomplete = false)],
)]
async fn vn_staff_command(self_: VnStaffCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let vndb_cache = cx.vndb_cache.clone();

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let staff = map
		.get(&String::from("name"))
		.cloned()
		.unwrap_or(String::new());

	let lang_id = cx.lang_id().await;

	let staff = get_staff(staff.clone(), vndb_cache.clone()).await?;

	let staff = staff.results[0].clone();

	let fields = vec![
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_staff-lang"),
			staff.lang,
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_staff-aid"),
			staff.aid.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_staff-gender"),
			staff.gender.clone().unwrap_or(String::new()),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_staff-main"),
			staff.ismain.to_string(),
			true,
		),
	];
	let staff_desc = staff.description.clone();

	let embed_content = EmbedContent::new(staff.name.clone())
		.description(String::from(convert_vndb_markdown(&staff_desc)))
		.fields(fields)
		.url(format!("https://vndb.org/{}", staff.id));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
