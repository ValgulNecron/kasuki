use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::command::{get_option_map_boolean, get_option_map_string};
use anyhow::anyhow;
use kasuki_macros::slash_command;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, IntoActiveModel};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::kill_switch::{ActiveModel, Column};
use shared::database::prelude::KillSwitch;
use shared::localization::{Loader, USABLE_LOCALES};
use small_fixed_array::FixedString;

#[slash_command(
	name = "kill_switch", desc = "Globally turn on or off a module",
	command_type = GuildChatInput { guild_id = 1117152661620408531 },
	permissions = [Administrator],
	args = [
		(name = "name", desc = "The module you want to change the state of.", arg_type = String, required = true, autocomplete = false,
			choices = [(name = "AI"), (name = "ANILIST"), (name = "GAME"), (name = "NEW_MEMBER"), (name = "ANIME"), (name = "VN")]),
		(name = "state", desc = "The state you want to to.", arg_type = Boolean, required = true, autocomplete = false)
	],
)]
async fn kill_switch_command(self_: KillSwitchCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string(&cx.command_interaction);

	let module = map
		.get(&FixedString::from_str_trunc("name"))
		.ok_or(anyhow!("No option for name"))?;

	let lang_id = cx.lang_id().await;

	let map = get_option_map_boolean(&cx.command_interaction);

	let state = *map
		.get(&FixedString::from_str_trunc("state"))
		.ok_or(anyhow!("No option for state"))?;

	let mut row = KillSwitch::find()
		.filter(Column::GuildId.eq("0"))
		.one(&*cx.db)
		.await?
		.ok_or(anyhow!("KillSwitch not found"))?;

	match module.as_str() {
		"ANILIST" => row.anilist_module = state,
		"AI" => row.ai_module = state,
		"GAME" => row.game_module = state,
		"LEVEL" => row.level_module = state,
		"MINIGAME" => row.mini_game_module = state,
		"ANIME" => row.anime_module = state,
		"VN" => row.vn_module = state,
		_ => {
			return Err(anyhow!("The module specified does not exist"));
		},
	}

	let active_model: ActiveModel = row.into_active_model();

	active_model.update(&*cx.db).await?;

	let desc = if state {
		USABLE_LOCALES.lookup(&lang_id, "management_kill_switch-on")
	} else {
		USABLE_LOCALES.lookup(&lang_id, "management_kill_switch-off")
	};

	let embed_content = EmbedContent::new(module.clone()).description(desc);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}
