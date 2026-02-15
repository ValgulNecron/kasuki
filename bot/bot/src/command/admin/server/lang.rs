//! The `LangCommand` struct handles the execution of a user command related
//! to changing the language settings for a guild (server). This struct
//! includes the context and command interaction necessary to process the command.
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::guild_lang;
use shared::database::prelude::GuildLang;
use shared::localization::USABLE_LOCALES;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

#[slash_command(
	name = "lang", desc = "Change the language of the bot's response.",
	command_type = SubCommandGroup(parent = "admin", group = "general"),
	args = [(name = "lang_choice", desc = "The language you want to set the response to.", arg_type = String, required = true, autocomplete = false,
		choices = [
			(name = "en"),
			(name = "jp"),
			(name = "de"),
			(name = "fr"),
			(name = "es-ES"),
			(name = "zh-CN"),
			(name = "ru")
		])],
)]
async fn lang_command(self_: LangCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let command_interaction = self_.get_command_interaction();
	let bot_data = ctx.data::<BotData>().clone();
	let db_connection = bot_data.db_connection.clone();

	let map = get_option_map_string_subcommand_group(command_interaction);
	let lang = map
		.get(&String::from("lang_choice"))
		.ok_or(anyhow!("No option for lang_choice"))?;

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	GuildLang::insert(guild_lang::ActiveModel {
		guild_id: Set(guild_id.clone()),
		lang: Set(lang.clone()),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(guild_lang::Column::GuildId)
			.update_column(guild_lang::Column::Lang)
			.to_owned(),
	)
	.exec(&*db_connection)
	.await?;

	let lang_code = match lang.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};

	let lang_id = LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());

	let mut args = HashMap::new();
	args.insert(Cow::Borrowed("lang"), FluentValue::from(lang.as_str()));

	let title = USABLE_LOCALES.lookup(&lang_id, "admin_server_lang-title");
	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "admin_server_lang-desc", &args);

	let embed_content = EmbedContent::new(title).description(desc);

	let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

	Ok(embed_contents)
}
