//! The `ModuleCommand` struct represents a command to manage module activations in a Discord bot.
//! It contains context and interaction details necessary for processing the command.
use crate::command::command::{Command, EmbedContent, EmbedType};
use crate::database::module_activation::Model;
use crate::database::prelude::ModuleActivation;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::{
	get_option_map_boolean_subcommand_group, get_option_map_string_subcommand_group,
};
use crate::structure::message::admin::server::module::load_localization_module_activation;
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};

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
pub struct ModuleCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ModuleCommand {
	/// Retrieves a reference to the `SerenityContext` associated with this instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` held within the instance.
	///
	/// # Usage
	/// This method provides access to the underlying `SerenityContext`, which can be used
	/// to interact with Discord's API through the Serenity library. It is commonly used
	/// when you need to perform actions such as sending messages, managing guilds, or
	/// retrieving other resources.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` to interact with the Discord API
	/// ```
	///
	/// # Notes
	/// This method borrows the context immutably, meaning you can safely call it
	/// within code that requires read-only access to the `SerenityContext`.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` stored within the current object.
	///
	/// # Example
	///
	/// ```
	/// let command_interaction = object.get_command_interaction();
	/// // Use the command_interaction as needed
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and updates the state of a specific module for a guild in a Discord bot context.
	///
	/// This function performs the following operations:
	/// 1. Gathers the execution context of the bot and extracts data such as the command interaction details,
	///    the database connection, and the guild ID.
	/// 2. Retrieves the options provided in the command for identifying the targeted module ("name") and its
	///    desired activation state ("state").
	/// 3. Loads localized text for module activation messages based on the guild ID.
	/// 4. Queries the current module activation settings for the guild from the database. If no settings are
	///    found, default settings are used.
	/// 5. Updates the activation status for the specific module based on the command input.
	/// 6. Saves the updated activation settings to the database.
	/// 7. Creates and returns an embed message to provide feedback to the user about the module activation
	///    status.
	///
	/// # Returns
	/// A `Result` containing a vector of `EmbedContent` objects for providing feedback about the operation. In the case
	/// of an error, an `anyhow::Error` is returned.
	///
	/// # Errors
	/// - Returns an error if the command interaction does not include a valid "name" option for the module.
	/// - Returns an error if the command interaction does not include a valid "state" option for the module.
	/// - Returns an error if the specified module name does not exist.
	/// - Returns an error if there are issues querying or updating the database.
	/// - Returns an error if there are problems with loading localized messages.
	///
	/// # Example
	/// ```rust
	/// let embed_contents = some_bot_object.get_contents().await?;
	/// for embed in embed_contents {
	///     // Send the embed as a response or perform other actions
	/// }
	/// ```
	///
	/// # Dependencies
	/// - Requires the `async-trait` attribute for asynchronous methods in traits.
	/// - Depends on the `sea-orm` library for database queries and updates.
	/// - Relies on a custom implementation of localization (`load_localization_module_activation`) and embed creation
	///   (`EmbedContent`).
	///
	/// # Notes
	/// - This function assumes the presence of a database schema for storing and managing module activation settings.
	/// - Default settings are used if no settings are found for the specified guild.
	/// - The embed message contents are localized based on the guild-specific language setting.
	///
	/// # Parameters
	/// - `self`: The instance of the struct or implementation that provides contextual information, including
	///           access to the database connection and command interaction details.
	///
	/// # Return Type
	/// - `Result<Vec<EmbedContent<'_, '_>>>`: A `Result` containing a `Vec` of embed content or an error.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		let bot_data = ctx.data::<BotData>().clone();
		let connection = bot_data.db_connection.clone();

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

		let module_localised =
			load_localization_module_activation(guild_id.clone(), bot_data.config.db.clone());

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

		let module_localised = module_localised.await?;
		let desc = if state {
			&module_localised.on
		} else {
			&module_localised.off
		};

		let embed_content = EmbedContent::new(module.clone())
			.description(desc.clone())
			.command_type(EmbedType::First);

		Ok(vec![embed_content])
	}
}

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
		"NEW_MEMBER" => row.new_members_module,
		"ANIME" => row.anime_module,
		"VN" => row.vn_module,
		_ => false,
	}
}
