//! The `LangCommand` struct handles the execution of a user command related
//! to changing the language settings for a guild (server). This struct
//! includes the context and command interaction necessary to process the command.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
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
use shared::localization::{available_locales, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

#[slash_command(
	name = "lang", desc = "Change the language of the bot's response.",
	command_type = SubCommandGroup(parent = "admin", group = "general"),
	args = [(name = "lang_choice", desc = "The language you want to set the response to.", arg_type = String, required = true, autocomplete = true)],
)]
async fn lang_command(self_: LangCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand_group(&cx.command_interaction);
	let lang = map
		.get("lang_choice")
		.ok_or(anyhow!("No option for lang_choice"))?;

	let locales = available_locales();
	if !locales.contains(lang) {
		return Err(anyhow!("Unknown language: {}", lang));
	}

	GuildLang::insert(guild_lang::ActiveModel {
		guild_id: Set(cx.guild_id.clone()),
		lang: Set(lang.clone()),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(guild_lang::Column::GuildId)
			.update_column(guild_lang::Column::Lang)
			.to_owned(),
	)
	.exec(&*cx.db)
	.await?;

	let lang_id = LanguageIdentifier::from_str(lang)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());

	let mut args = HashMap::new();
	args.insert(Cow::Borrowed("lang"), FluentValue::from(lang.as_str()));

	let title = USABLE_LOCALES.lookup(&lang_id, "admin_server_lang-title");
	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "admin_server_lang-desc", &args);

	let embed_content = EmbedContent::new(title).description(desc);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
