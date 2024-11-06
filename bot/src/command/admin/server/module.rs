use crate::command::command_trait::{Command, Embed, EmbedType, SlashCommand};
use crate::database::module_activation::Model;
use crate::database::prelude::ModuleActivation;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand_group::{
	get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::structure::message::admin::server::module::load_localization_module_activation;
use anyhow::{anyhow, Result};
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};

pub struct ModuleCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ModuleCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for ModuleCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = &self.ctx;
		let command_interaction = &self.command_interaction;
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let bot_data = ctx.data::<BotData>().clone();

		let map = get_option_map_string_subcommand_group(command_interaction);

		let module = map
			.get(&String::from("name"))
			.ok_or(anyhow!("No option for name"))?;

		let module_localised =
			load_localization_module_activation(guild_id.clone(), bot_data.config.db.clone())
				.await?;

		let map = get_option_map_boolean_subcommand_group(command_interaction);

		let state = *map
			.get(&String::from("state"))
			.ok_or(anyhow!("No option for state"))?;

		let mut row = ModuleActivation::find()
			.filter(crate::database::module_activation::Column::GuildId.eq(guild_id.clone()))
			.one(&*connection)
			.await?
			.unwrap_or(Model {
				guild_id,
				ai_module: true,
				anilist_module: true,
				game_module: true,
				new_members_module: false,
				anime_module: true,
				vn_module: true,
				updated_at: Default::default(),
			});

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

		let active_model = row.into_active_model();

		active_model.update(&*connection).await?;

		let desc = if state {
			&module_localised.on
		} else {
			&module_localised.off
		};

		self.send_embed(
			Vec::new(),
			None,
			module.clone(),
			desc.clone(),
			None,
			None,
			EmbedType::First,
			None,
			Vec::new(),
		)
		.await
	}
}

pub async fn check_activation_status(module: &str, row: Model) -> bool {
	match module {
		"ANILIST" => row.anilist_module,
		"AI" => row.ai_module,
		"GAME" => row.game_module,
		"NEW_MEMBER" => row.new_members_module,
		"ANIME" => row.anime_module,
		"VN" => row.vn_module,
		_ => false,
	}
}
