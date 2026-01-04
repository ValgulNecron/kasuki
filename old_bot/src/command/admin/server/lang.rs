//! The `LangCommand` struct handles the execution of a user command related
//! to changing the language settings for a guild (server). This struct
//! includes the context and command interaction necessary to process the command.
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::guild_lang;
use crate::database::prelude::GuildLang;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::impl_command;
use crate::structure::message::admin::server::lang::load_localization_lang;
use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// The `LangCommand` struct is used to represent language-related commands within a Discord bot context.
///
/// It encapsulates the necessary context and interaction information for executing commands.
///
/// # Fields
///
/// * `ctx` (`SerenityContext`) - The context of the current Discord bot, including runtime data,
/// such as HTTP, cache, and shard states, required for executing actions or responding to events.
///
/// * `command_interaction` (`CommandInteraction`) - The interaction data for the command being executed,
/// containing information such as the invoking user, the channel the command was triggered in,
/// and the content of the command itself.
#[derive(Clone)]
pub struct LangCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for LangCommand,
	get_contents = |self_: LangCommand| async move {
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
		.exec(&*db_connection)
		.await?;

		let lang_localised = load_localization_lang(guild_id, db_connection).await?;

		let embed_content = EmbedContent::new(lang_localised.title.clone())
			.description(lang_localised.desc.replace("$lang$", lang.as_str()));

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
);
