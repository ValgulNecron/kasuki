//! The `LangCommand` struct handles the execution of a user command related
//! to changing the language settings for a guild (server). This struct 
//! includes the context and command interaction necessary to process the command.
use crate::command::command_trait::{Command, EmbedContent, EmbedType};
use crate::database::guild_lang;
use crate::database::prelude::GuildLang;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::server::lang::load_localization_lang;
use anyhow::{Result, anyhow};
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
pub struct LangCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LangCommand {
	/// Returns a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A shared reference to the `SerenityContext`.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the retrieved context here
	/// ```
	///
	/// # Notes
	/// This method provides access to the context (`self.ctx`) which is typically used 
	/// for interacting with Discord through the Serenity library.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance stored within the struct.
	///
	/// # Example
	/// ```
	/// let command_interaction = instance.get_command_interaction();
	/// // Use the command_interaction as needed
	/// ```
	///
	/// This method provides read-only access to the `command_interaction` field of the struct,
	/// allowing the caller to utilize it without modifying its contents.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes embed content.
	///
	/// This function performs several operations to retrieve and return a vector
	/// of `EmbedContent` structures, which are used for generating embeds that
	/// are localized and relevant to the specific context and options provided
	/// by the command interaction.
	///
	/// ## Steps Performed
	/// 1. Retrieves the context (`Ctx`) and command interaction for processing logical flow.
	/// 2. Extracts a specific language choice (`lang_choice`) from the options map.
	/// 3. Determines the guild ID or defaults to `"0"` if none is found.
	/// 4. Saves or updates the chosen language for the guild in the database.
	/// 5. Loads the localized content based on the guild's language settings.
	/// 6. Constructs and returns the resulting `EmbedContent` containing the localized details.
	///
	/// ## Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)`:
	///   A vector of `EmbedContent` objects containing the localized embed details on success.
	/// - `Err(Error)`:
	///   Returns an error if any step (e.g., missing `lang_choice`, database operation failure) fails.
	///
	/// ## Errors
	/// The function will return an `Err` in the following cases:
	/// - The "lang_choice" option is missing.
	/// - Any failure while interacting with the database for insertion/updating.
	/// - Errors during the localization loading process.
	///
	/// ## Example
	/// ```rust
	/// let contents = command.get_contents().await?;
	/// for content in contents {
	///     send_embed(content).await?;
	/// }
	/// ```
	///
	/// ## Remarks
	/// - This function interacts with the database to store guild language preferences,
	///   hence requires proper database connectivity.
	/// - The localization loading step is dependent on the seeded translation configuration.
	///
	/// ## Dependencies
	/// - `get_option_map_string_subcommand_group`: Retrieves a map of options for the subcommand interactions.
	/// - `GuildLang::insert`: Handles the insertion of language preferences in the guild database table.
	/// - `load_localization_lang`: Loads the localized content string based on the specified guild ID and language.
	///
	/// ## Arguments
	/// No explicit arguments are passed.
	/// The function relies on `self` to access the context and command interaction.
	///
	/// ## Panics
	/// None. Any failures are propagated using the `Result` type.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
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

		let embed_content = EmbedContent::new(lang_localised.title.clone())
			.description(lang_localised.desc.replace("$lang$", lang.as_str()))
			.command_type(EmbedType::First);

		Ok(vec![embed_content])
	}
}
