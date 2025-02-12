use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::database::guild_lang;
use crate::database::prelude::GuildLang;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::server::lang::load_localization_lang;
use anyhow::{anyhow, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};

pub struct LangCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LangCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for LangCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = &self.ctx;
		let command_interaction = &self.command_interaction;
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();

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
		.exec(&*connection)
		.await?;

		let lang_localised = load_localization_lang(guild_id, bot_data.config.db.clone()).await?;

		let embed_content = EmbedContent {
			title: lang_localised.title.clone(),
			description: lang_localised.desc.replace("$lang$", lang.as_str()),
			thumbnail: None,
			url: None,
			command_type: EmbedType::First,
			colour: None,
			fields: vec![],
			images: None,
			action_row: None,
		};

		self.send_embed(embed_content).await
	}
}
