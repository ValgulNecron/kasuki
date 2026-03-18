//! The `ModuleCommand` struct represents a command to manage module activations in a Discord bot.
//! It contains context and interaction details necessary for processing the command.
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand_group::{
	get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use anyhow::anyhow;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use sea_orm::{ColumnTrait, Set};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::module_activation;
use shared::database::prelude::ModuleActivation;
use shared::localization::USABLE_LOCALES;
use tracing::debug;

#[slash_command(
	name = "module", desc = "Turn on or off a module.",
	command_type = SubCommandGroup(parent = "admin", group = "general"),
	args = [
		(name = "name", desc = "The module you want to change the state of.", arg_type = String, required = true, autocomplete = false,
			choices = [
				(name = "AI"),
				(name = "ANILIST"),
				(name = "GAME"),
				(name = "ANIME"),
				(name = "VN"),
				(name = "LEVEL"),
				(name = "MINIGAME")
			]),
		(name = "state", desc = "The state you want to to.", arg_type = Boolean, required = true, autocomplete = false)
	],
)]
async fn module_command(self_: ModuleCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand_group(&cx.command_interaction);
	let module = map.get("name").ok_or(anyhow!("No option for name"))?;

	let map = get_option_map_boolean_subcommand_group(&cx.command_interaction);
	let state = *map.get("state").ok_or(anyhow!("No option for state"))?;

	let lang_id = cx.lang_id().await;

	match ModuleActivation::find()
		.filter(module_activation::Column::GuildId.eq(cx.guild_id.clone()))
		.one(&*cx.db)
		.await?
	{
		None => {
			debug!(
				"No module activation found for guild {}. Creating new one.",
				cx.guild_id
			);
			let mut models = module_activation::ActiveModel {
				guild_id: Set(cx.guild_id.clone()),
				ai_module: Set(true),
				anilist_module: Set(true),
				game_module: Set(true),
				anime_module: Set(true),
				vn_module: Set(true),
				updated_at: Set(Default::default()),
				level_module: Set(false),
				mini_game_module: Set(true),
			};
			match module.as_str() {
				"ANILIST" => models.anilist_module = Set(state),
				"AI" => models.ai_module = Set(state),
				"GAME" => models.game_module = Set(state),
				"ANIME" => models.anime_module = Set(state),
				"VN" => models.vn_module = Set(state),
				"LEVEL" => models.level_module = Set(state),
				"MINIGAME" => models.mini_game_module = Set(state),
				_ => {
					return Err(anyhow!("The module specified does not exist"));
				},
			}
			ModuleActivation::insert(models)
				.on_conflict(
					sea_orm::sea_query::OnConflict::columns([module_activation::Column::GuildId])
						.do_nothing()
						.to_owned(),
				)
				.exec(&*cx.db)
				.await?;
		},
		Some(mut row) => {
			match module.as_str() {
				"ANILIST" => row.anilist_module = state,
				"AI" => row.ai_module = state,
				"GAME" => row.game_module = state,
				"ANIME" => row.anime_module = state,
				"VN" => row.vn_module = state,
				"LEVEL" => row.level_module = state,
				"MINIGAME" => row.mini_game_module = state,
				_ => {
					return Err(anyhow!("The module specified does not exist"));
				},
			}
			let active_model = row.into_active_model();
			active_model.update(&*cx.db).await?;
		},
	}

	let desc = if state {
		USABLE_LOCALES.lookup(&lang_id, "admin_server_module-on")
	} else {
		USABLE_LOCALES.lookup(&lang_id, "admin_server_module-off")
	};

	let embed_content = EmbedContent::new(module.clone()).description(desc);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
