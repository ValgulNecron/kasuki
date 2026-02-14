use anyhow::anyhow;
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::user::get_user;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
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
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let vndb_cache = bot_data.vndb_cache.clone();

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let map = get_option_map_string_subcommand(command_interaction);

	let user = map
		.get(&String::from("username"))
		.ok_or(anyhow!("No username provided"))?;

	let path = format!("/user?q={}&fields=lengthvotes,lengthvotes_sum", user);

	let user = get_user(path, vndb_cache).await?;
	let db_connection = bot_data.db_connection.clone();

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let fields = vec![
		(USABLE_LOCALES.lookup(&lang_id, "vn_user-id"), user.id.clone(), true),
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
		(USABLE_LOCALES.lookup(&lang_id, "vn_user-name"), user.username.clone(), true),
	];

	let mut title_args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
	title_args.insert(Cow::Borrowed("user"), FluentValue::from(user.username.clone()));
	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup_with_args(&lang_id, "vn_user-title", &title_args))
			.fields(fields);

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}
