use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::database::kill_switch::{ActiveModel, Column};
use crate::database::prelude::KillSwitch;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::command::{get_option_map_boolean, get_option_map_string};
use crate::structure::message::management::kill_switch::load_localization_kill_switch;
use anyhow::{anyhow, Result};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{EntityTrait, IntoActiveModel};
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use small_fixed_array::FixedString;
use std::sync::Arc;

pub struct KillSwitchCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for KillSwitchCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for KillSwitchCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		send_embed(
			&self.ctx,
			&self.command_interaction,
			bot_data.config.clone(),
		)
		.await
	}
}

async fn send_embed(
	ctx: &SerenityContext, command_interaction: &CommandInteraction, config: Arc<Config>,
) -> Result<()> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let map = get_option_map_string(command_interaction);

	let module = map
		.get(&FixedString::from_str_trunc("name"))
		.ok_or(anyhow!("No option for name"))?;

	let module_localised =
		load_localization_kill_switch(guild_id.clone(), config.db.clone()).await?;

	let map = get_option_map_boolean(command_interaction);

	let state = *map
		.get(&FixedString::from_str_trunc("state"))
		.ok_or(anyhow!("No option for state"))?;

	let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

	let mut row = KillSwitch::find()
		.filter(Column::GuildId.eq("0"))
		.one(&connection)
		.await?
		.ok_or(anyhow!("KillSwitch not found"))?;

	match module.as_str() {
		"ANILIST" => row.anilist_module = state,
		"AI" => row.ai_module = state,
		"GAME" => row.game_module = state,
		"NEW_MEMBER" => row.new_members_module = state,
		"ANIME" => row.anime_module = state,
		"VN" => row.vn_module = state,
		_ => {
			return Err(anyhow!("The module specified does not exist"));
		},
	}

	let active_model: ActiveModel = row.into_active_model();

	active_model.update(&connection).await?;

	let desc = if state {
		&module_localised.on
	} else {
		&module_localised.off
	};

	let builder_embed = get_default_embed(None).description(desc).title(module);

	let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

	let builder = CreateInteractionResponse::Message(builder_message);

	command_interaction
		.create_response(&ctx.http, builder)
		.await?;

	Ok(())
}
