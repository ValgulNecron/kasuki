use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::context::CommandContext;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use shared::vndb::user::get_user;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "user", desc = "Get info of a VN user.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the VN user.", arg_type = String, required = true, autocomplete = false)],
)]
async fn vn_user_command(self_: VnUserCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let cx = CommandContext::new(self_.get_ctx().clone(), self_.get_command_interaction().clone());
	let vndb_cache = cx.vndb_cache.clone();

	let map = get_option_map_string_subcommand(&cx.command_interaction);

	let user = map
		.get(&String::from("username"))
		.ok_or(anyhow!("No username provided"))?;

	let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);

	let user = get_user(path, vndb_cache).await?;

	let lang_id = cx.lang_id().await;

	let fields = vec![
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_user-id"),
			user.id.clone(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_user-playtime"),
			user.lengthvotes.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_user-playtimesum"),
			user.lengthvotes_sum.to_string(),
			true,
		),
		(
			USABLE_LOCALES.lookup(&lang_id, "vn_user-name"),
			user.username.clone(),
			true,
		),
	];

	let mut title_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	title_args.insert(
		Cow::Borrowed("user"),
		FluentValue::from(user.username.clone()),
	);
	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup_with_args(&lang_id, "vn_user-title", &title_args))
			.fields(fields);

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}
