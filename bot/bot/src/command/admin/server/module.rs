
//! The `ModuleCommand` struct represents a command to manage module activations in a Discord bot.
//! It contains context and interaction details necessary for processing the command.
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::{
	get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::impl_command;
use anyhow::anyhow;
use fluent_templates::Loader;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use sea_orm::{ColumnTrait, Set};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::module_activation;
use shared::database::module_activation::Model;
use shared::database::prelude::ModuleActivation;
use shared::localization::{get_language_identifier, USABLE_LOCALES};
use tracing::debug;

/// A structure representing a command executed within a module,
/// encapsulating the context and details of the command interaction.
///
/// # Fields
///
/// * `ctx` - Represents the current context of the Serenity framework, providing
///           access to features such as interacting with Discord's API and managing state.
///
/// * `command_interaction` - Contains the information and details about the command interaction
///                           that was executed by a user, such as the input data and the source of the interaction.
///
/// # Usage
///
/// The `ModuleCommand` struct is used to package all relevant
/// information needed to handle a specific command interaction
/// within a module in a bot powered by the Serenity library.
#[derive(Clone)]
pub struct ModuleCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for ModuleCommand,
	get_contents = |self_: ModuleCommand| async move {
		let ctx = self_.get_ctx();
		let command_interaction = self_.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let db_connection = bot_data.db_connection.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let map = get_option_map_string_subcommand_group(command_interaction);
		let module = map
			.get(&String::from("name"))
			.ok_or(anyhow!("No option for name"))?;

		let map = get_option_map_boolean_subcommand_group(command_interaction);
		let state = *map
			.get(&String::from("state"))
			.ok_or(anyhow!("No option for state"))?;

		let lang_id = get_language_identifier(guild_id.clone(), db_connection.clone()).await;

		match ModuleActivation::find()
			.filter(module_activation::Column::GuildId.eq(guild_id.clone()))
			.one(&*db_connection)
			.await?
		{
			None => {
			debug!("No module activation found for guild {}. Creating new one.", guild_id);
			let mut models = module_activation::ActiveModel {
					guild_id: Set(guild_id),
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
				.exec(&*db_connection.clone())
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
				active_model.update(&*db_connection).await?;
			},
		}

		let desc = if state {
			USABLE_LOCALES.lookup(&lang_id, "admin_server_module-on")
		} else {
			USABLE_LOCALES.lookup(&lang_id, "admin_server_module-off")
		};

		let embed_content = EmbedContent::new(module.clone()).description(desc);

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
);

/// Checks the activation status of a specific module based on the provided `module` name and the associated `row` data.
///
/// # Arguments
///
/// * `module` - A string slice that specifies the name of the module to be checked. The valid values are:
///     - "ANILIST"
///     - "AI"
///     - "GAME"
///     - "NEW_MEMBER"
///     - "ANIME"
///     - "VN"
/// * `row` - A `Model` instance containing the activation statuses of various modules as boolean attributes.
///
/// # Returns
///
/// A boolean indicating the activation status of the specified module:
///
/// * `true` - The module is activated.
/// * `false` - The module is not activated or the `module` name does not match any valid options.
///
/// # Example
///
/// ```rust
/// let model = Model {
///     anilist_module: true,
///     ai_module: false,
///     game_module: true,
///     new_members_module: false,
///     anime_module: true,
///     vn_module: false,
/// };
///
/// let is_active = check_activation_status("ANILIST", model).await;
/// assert_eq!(is_active, true);
///
/// let is_ai_active = check_activation_status("AI", model).await;
/// assert_eq!(is_ai_active, false);
///
/// let is_invalid_active = check_activation_status("INVALID_MODULE", model).await;
/// assert_eq!(is_invalid_active, false);
/// ```
///
/// # Note
/// This function returns `false` for any module name that does not match the predefined list of modules.
pub async fn check_activation_status(module: &str, row: Model) -> bool {
	match module {
		"ANILIST" => row.anilist_module,
		"AI" => row.ai_module,
		"GAME" => row.game_module,
		"ANIME" => row.anime_module,
		"VN" => row.vn_module,
		"LEVEL" => row.level_module,
		"MINIGAME" => row.mini_game_module,
		_ => false,
	}
}
