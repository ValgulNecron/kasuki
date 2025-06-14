//! The `ProfileCommand` struct represents a command for handling and displaying user profile
//! information in a Discord bot. It contains the Serenity context and the interaction data for
//! processing and responding to the user command.
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::event_handler::BotData;
use crate::structure::message::user::profile::{ProfileLocalised, load_localization_profile};
use serenity::all::{CommandInteraction, Context as SerenityContext, Member, User};

/// `ProfileCommand` represents a structure that handles the context and interaction
/// details required for processing a profile-related command in a Discord bot.
///
/// # Fields
///
/// * `ctx` - The context of the bot, represented by `SerenityContext`.
///   This provides access to various utilities and information about the bot's state,
///   such as shard info, data storage, and HTTP operations.
///
/// * `command_interaction` - Contains data related to the specific command interaction
///   issued by the user. This includes details such as the user who invoked the command,
///   options provided, and the originating message.
///
/// # Usage
///
/// This struct is intended to encapsulate the necessary data for handling a profile-related
/// command in a modular manner, enabling easier management, execution, or processing
/// of commands associated with user profiles in a Discord bot.
///
/// # Example
///
/// ```rust
/// let profile_command = ProfileCommand {
///     ctx,
///     command_interaction,
/// };
/// // Process the profile-related command using `profile_command`
/// ```
pub struct ProfileCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ProfileCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored in the current instance.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` for operations requiring the Serenity context.
	/// ```
	///
	/// This method is typically used to access the context object needed to interact with Discord
	/// through the Serenity library.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to a `CommandInteraction` object that represents the interaction command
	/// linked to the instance.
	///
	/// # Examples
	/// ```
	/// let interaction = instance.get_command_interaction();
	/// ```
	/// Use this method when you need to access the `CommandInteraction` without taking ownership of it.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves user-related embed contents for commands, with localized profile information, user data,
	/// server membership details, and premium entitlement information.
	///
	/// # Returns
	///
	/// Returns a `Result` containing either the successfully constructed `EmbedsContents`,
	/// or an error if any step in the process fails.
	///
	/// # Process
	///
	/// 1. **Extract Context and Dependencies**: Retrieves the relevant context and configuration data.
	/// 2. **Determine Command Type**: Matches the command type to fetch the user information and determine the targeted user.
	/// 3. **Load Guild-Specific Localization Profile**: If the interaction occurs in a guild, retrieves the localized profile
	///    data for user embedding based on the guild ID.
	/// 4. **Construct Embed Fields**:
	///     - Adds user information fields based on localized settings.
	///     - If applicable, adds the join date for guild members.
	/// 5. **Fetch Premium Entitlements**:
	///     - Obtains available SKUs (Stock Keeping Units) for premium content.
	///     - Fetches and associates premium entitlements for the targeted user.
	///     - Constructs a formatted string listing entitlement details, including premium types and timelines.
	///     - Adds this information to the embed fields.
	/// 6. **Build Embed Content**:
	///     - Creates embedded content with all available fields, localized titles, avatars, banners, and additional details.
	/// 7. **Return Results**:
	///     - Packages the embed content into an `EmbedsContents` object and returns it.
	///
	/// # Errors
	///
	/// This method can fail under the following circumstances:
	/// - Invalid command type provided (not matching known cases).
	/// - Failure to fetch user details, guild membership, localization profile, SKUs, or entitlements.
	/// - Other internal errors related to API calls or data access.
	///
	/// # Example
	///
	/// ```rust
	/// let embed_contents = your_instance.get_contents().await?;
	/// // Use embed_contents as needed, such as sending responses in a bot interaction.
	/// ```
	///
	/// # Note
	///
	/// This function handles both user commands (`kind == 1`) and command-user scenarios (`kind == 2`).
	/// It is designed specifically for Discord bots orchestrated in a multi-guild context,
	/// ensuring proper localized responses and detailed user insights.
	///
	/// Dependencies:
	/// - `self.get_ctx()`: Retrieves the bot's execution context.
	/// - `self.get_command_interaction()`: Accesses the current command interaction handling instance.
	/// - `load_localization_profile()` and `get_fields()`: Localizes user details and formats them according to guild-specific settings.
	/// - API calls for fetching SKUs, premium entitlements, and guild membership details.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let db_config = bot_data.config.db.clone();

		let user = match command_interaction.data.kind.0 {
			1 => get_user_command(ctx, command_interaction).await?,
			2 => get_user_command_user(ctx, command_interaction).await,
			_ => {
				return Err(anyhow::anyhow!("Invalid command type"));
			},
		};

		let guild_id = command_interaction
			.guild_id
			.map(|id| id.to_string())
			.unwrap_or("0".to_string());

		let profile_localised = load_localization_profile(guild_id, db_config).await?;

		let mut fields = get_fields(&profile_localised, user.clone());

		let avatar_url = user.face();

		let member: Option<Member> = {
			match command_interaction.guild_id {
				Some(guild_id) => (guild_id.member(&ctx.http, user.id).await).ok(),
				None => None,
			}
		};

		if let Some(member) = member {
			if let Some(joined_at) = member.joined_at {
				fields.push((
					profile_localised.joined_date,
					format!("<t:{}>", joined_at.timestamp()),
					true,
				));
			}
		}

		let skus = ctx.http.get_skus().await;

		let user_premium = ctx
			.http
			.get_entitlements(Some(user.id), None, None, None, None, None, Some(true))
			.await;

		if user_premium.is_ok() && skus.is_ok() {
			let skus = skus?.clone();

			let data = user_premium?;

			if !data.is_empty() {
				let string = data.iter().map(|e| {
					let sku_id = e.sku_id;

					let sku = skus.iter().find(|e2| e2.id == sku_id);

					let e_type = e.kind.clone();
					let type_name = match e_type.0 {
						8 => String::from("APPLICATION_SUBSCRIPTION"),
						1 => String::from("purchase"),
						2 => String::from("premium_subscription"),
						3 => String::from("developer_gift"),
						4 => String::from("test_mode_purchase"),
						5 => String::from("free_purchase"),
						6 => String::from("user_gift"),
						7 => String::from("premium_purchase"),
						_ => String::from("Unknown"),
					};

					let sku_name = match sku {
						Some(sku) => sku.name.clone(),
						None => String::from("Unknown"),
					};

					format!(
						"{}: {}/{} \n {}",
						sku_name,
						e.starts_at.unwrap_or_default(),
						e.ends_at.unwrap_or_default(),
						type_name
					)
				});

				fields.push((profile_localised.premium, string.collect::<String>(), true));
			}
		}
		let embed_content = EmbedContent::new(
			profile_localised
				.title
				.replace("$user$", user.name.as_str()),
		)
		.thumbnail(avatar_url)
		.fields(fields)
		.images_url(user.banner_url().unwrap_or_default());

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}

/// Generates a vector of fields containing information extracted from a user's profile.
///
/// # Parameters
///
/// - `profile_localised`: A reference to a [`ProfileLocalised`] structure that provides localized
///   field names or keys (e.g., for ID, creation date, bot status, etc.).
/// - `user`: A [`User`] object containing user-specific information to populate the fields.
///
/// # Returns
///
/// Returns a `Vec` of tuples `(String, String, bool)`, where each tuple represents:
/// - The field name (localized, as in `profile_localised`),
/// - The field value (derived from the `user` object),
/// - A boolean indicating whether the field is inline (`true`) or not (`false`).
///
/// The default fields include:
/// - User ID
/// - Account creation date (formatted as a timestamp for rendering)
/// - Whether the user is a bot
/// - Whether the user is part of a system
///
/// Additionally, if the user has public flags (accessible via `user.public_flags`):
/// - The flags are retrieved, formatted, and added as a new field.
///
/// # Example Output
///
/// An example of a single field in the vector:
///
/// ```text
/// (
///   "ID".to_string(),                     // Localized field name
///   "123456789".to_string(),              // User ID as string
///   true,                                 // Inline status
/// )
/// ```
///
/// # Notes
///
/// - The `user.public_flags` field is optional. If it exists, the user's flags
///   are iterated, converted to strings, and joined with " / " separators.
/// - Any added flag field will have `false` for the inline status.
///
/// # Dependencies
///
/// This function requires:
/// - A `ProfileLocalised` implementation to provide localized field names.
/// - The `User` structure, which should include:
///   - User ID (`id`), and a method `created_at()` to get a creation timestamp.
///   - Whether the user is a bot or part of a system (`bot()`/`system()` methods).
///   - Optional public flags (`public_flags`) with an `iter_names()` method.
fn get_fields(profile_localised: &ProfileLocalised, user: User) -> Vec<(String, String, bool)> {
	let mut fields = vec![
		(
			profile_localised.id.clone(),
			user.id.clone().to_string(),
			true,
		),
		(
			profile_localised.creation_date.clone(),
			format!("<t:{}>", user.id.created_at().timestamp()),
			true,
		),
		(profile_localised.bot.clone(), user.bot().to_string(), true),
		(
			profile_localised.system.clone(),
			user.system().to_string(),
			true,
		),
	];

	if let Some(public_flag) = user.public_flags {
		let mut user_flags = Vec::new();

		// If there are, iterate over the flags and add them to a vector
		for (flag, _) in public_flag.iter_names() {
			user_flags.push(flag)
		}

		if !user_flags.is_empty() {
			fields.push((
				profile_localised.public_flag.clone(),
				user_flags.join(" / "),
				false,
			));
		}
	}

	fields
}
