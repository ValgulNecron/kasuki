//! # StaffCommand Module
//!
//! This module defines the `StaffCommand` struct and implements the required functionality
//! to retrieve and format staff information from the AniList GraphQL API. It includes
//! methods for extracting staff details such as roles, media appearances, and other metadata
//! to be formatted into Discord Embed responses.
use std::sync::Arc;

use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::staff::load_localization_staff;
use crate::structure::run::anilist::staff::{
	FuzzyDate, Staff, StaffQuerryId, StaffQuerryIdVariables, StaffQuerrySearch,
	StaffQuerrySearchVariables,
};
use anyhow::{Result, anyhow};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

/// A structure representing a staff command within the bot application.
///
/// This structure is designed to encapsulate the relevant context and interaction
/// details for a given command issued by staff members. It provides an organized way
/// to handle and process staff-specific commands in conjunction with the Discord API.
///
/// # Fields
///
/// * `ctx` - A `SerenityContext` object that provides access to the bot's runtime data,
///           such as the shard manager, cache, and HTTP functionalities required to handle
///           the interactions.
/// * `command_interaction` - A `CommandInteraction` object representing the interaction details.
///                           It contains information about the command issued by the user,
///                           such as the command name, arguments, and other metadata.
///
/// # Example
///
/// ```rust
/// let staff_command = StaffCommand {
///     ctx: serenity_context,
///     command_interaction: interaction
/// };
/// // Proceed to use `staff_command` for handling the interaction
/// ```
///
/// This structure is particularly useful for modularizing and organizing staff commands
/// to enhance maintainability and scalability of bot functionalities.
pub struct StaffCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for StaffCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current object.
	///
	/// # Returns
	/// A reference to the `SerenityContext` instance.
	///
	/// # Example
	/// ```rust
	/// let context = my_object.get_ctx();
	/// // Now you can use `context` to interact with the Serenity API.
	/// ```
	///
	/// This method allows access to the context, which can be used
	/// for interacting with the Serenity framework, such as sending messages,
	/// fetching data, or interacting with Discord directly.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` stored within the instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// // Use the `command_interaction` as needed
	/// ```
	///
	/// This function allows access to the `command_interaction` field without consuming the instance.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves embed contents based on staff data from AniList.
	///
	/// # Returns
	/// A `Result` containing either:
	/// - A `Vec<EmbedContent<'_, '_>>` representing the embed contents, or
	/// - An error if the operation fails.
	///
	/// # Process
	/// 1. Fetches the context and command interaction.
	/// 2. Retrieves required data such as configuration, AniList cache, and staff information.
	/// 3. Parses the staff details (e.g., characters, media appearances, occupation, gender, etc.) into a structured format.
	/// 4. Loads localized data based on the guild ID and applies it to the embed fields.
	/// 5. Assembles the structured staff information into an embed content object (`EmbedContent`).
	///
	/// # Fields Populated
	/// - Media associated with the staff (e.g., anime titles the staff worked on).
	/// - Primary occupation of the staff.
	/// - Gender of the staff member.
	/// - Language spoken by the staff member.
	/// - Optional fields (if available):
	///   - Hometown
	///   - Voice acting roles
	///   - Age
	///   - Date of birth
	///   - Date of death
	///
	/// # Error Handling
	/// The method may return an error in cases such as:
	/// - Failure to fetch staff data from AniList.
	/// - Failure to process localization data.
	///
	/// # Examples
	/// ```no_run
	/// let embed_contents = my_instance.get_contents().await.unwrap();
	/// for embed in embed_contents {
	///     // Process or send the embed content
	/// }
	/// ```
	///
	/// # Dependencies
	/// - This function relies on the `get_staff` function to fetch staff data.
	/// - Requires helper functions like `get_full_name`, `get_date`, and localization loading utilities.
	///
	/// # Note
	/// This function assumes that staff data and localization data are well-formed and available.
	/// Unavailable or malformed data will be replaced with fallback values (e.g., "Unknown").
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let config = bot_data.config.clone();

		let anilist_cache = bot_data.anilist_cache.clone();
		let staff = get_staff(command_interaction, anilist_cache).await?;

		let va = staff
			.characters
			.unwrap()
			.nodes
			.unwrap()
			.iter()
			.filter_map(|x| {
				let x = x.clone().unwrap();
				let name = x.name.unwrap();
				let full = name.full.as_deref();
				let native = name.native.as_deref();
				get_full_name(full, native)
			})
			.take(5)
			.collect::<Vec<String>>()
			.join("\n");

		let media = staff
			.staff_media
			.unwrap()
			.edges
			.unwrap()
			.iter()
			.filter_map(|x| {
				let node = x.clone().unwrap().node.unwrap();
				let title = node.title.unwrap();
				let romaji = title.romaji.as_deref();
				let english = title.english.as_deref();
				get_full_name(romaji, english)
			})
			.take(5)
			.collect::<Vec<String>>()
			.join("\n");

		let job = staff.primary_occupations.unwrap()[0]
			.clone()
			.unwrap_or_default();

		let gender = staff.gender.clone().unwrap_or(String::from("Unknown."));

		let lang = staff.language_v2.unwrap_or_default();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};
		let staff_localised = load_localization_staff(guild_id, config.db.clone()).await?;

		let mut fields = vec![
			(staff_localised.media, media, true),
			(staff_localised.occupation, job, true),
			(staff_localised.gender, gender, true),
			(staff_localised.lang, lang, true),
		];

		if let Some(home_town) = staff.home_town {
			fields.push((staff_localised.hometown, home_town, true))
		}

		if !va.is_empty() {
			fields.push((staff_localised.va, va, true))
		}

		let age = staff.age;

		if age.is_some() {
			fields.push((
				staff_localised.age,
				age.unwrap_or_default().to_string(),
				true,
			))
		}

		let name = staff.name.unwrap();
		if staff.date_of_birth.is_some() {
			let date_of_birth = get_date(staff.date_of_birth.clone());
			if date_of_birth != String::new() {
				fields.push((staff_localised.date_of_birth, date_of_birth, true));
			}
		}

		if staff.date_of_death.is_some() {
			let date_of_death = get_date(staff.date_of_death.clone());
			if date_of_death != String::new() {
				fields.push((staff_localised.date_of_death, date_of_death, true));
			}
		}

		let name = name.full.unwrap_or(
			name.user_preferred
				.unwrap_or(name.native.unwrap_or(String::from("Unknown."))),
		);

		let embed_content = EmbedContent::new(name)
			.description(convert_anilist_flavored_to_discord_flavored_markdown(
				staff.description.unwrap_or_default(),
			))
			.thumbnail(staff.image.unwrap().large)
			.url(staff.site_url)
			.command_type(EmbedType::First)
			.fields(fields);

		Ok(vec![embed_content])
	}
}

/// Retrieves staff details from the AniList API based on the provided command interaction.
///
/// This function uses either the staff ID or staff search query provided in the command
/// interaction to fetch the corresponding staff information from AniList.
///
/// # Arguments
///
/// * `command_interaction` - A reference to the [`CommandInteraction`] object that contains
///   details about the command executed, including options or parameters.
///
/// * `anilist_cache` - A thread-safe reference to an AniList cache of type [`Cache`],
///   used to store and retrieve previously retrieved data to minimize unnecessary API calls.
///
/// # Returns
///
/// On success, this returns a [`Result`] wrapping the [`Staff`] object containing the
/// details of the staff queried.
///
/// On failure, this returns an [`Err`] with a context-specific error.
///
/// # How It Works
///
/// 1. Options from the `command_interaction` are parsed into a map.
/// 2. The function checks for the presence of the `staff_name` option. If not found,
///    it returns an error.
/// 3. If the provided staff name can be parsed into an integer (`i32`), it treats the input
///    as a staff ID and fetches the data using the `StaffQuerryId` query.
/// 4. If the input cannot be parsed into an integer, it assumes the input is a name and
///    fetches the relevant staff data using the `StaffQuerrySearch` query.
/// 5. Makes use of the `make_request_anilist` function to send GraphQL requests for data fetching.
/// 6. Extracts and returns the staff information upon successful query execution.
///
/// # Errors
///
/// This function returns an error in the following scenarios:
///
/// * No `staff_name` is specified in the command interaction.
/// * The provided `staff_name` cannot be successfully parsed or matched in the AniList API.
/// * Any network, parsing, or GraphQL execution failure occurs during the data retrieval process.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use my_project::{get_staff, CommandInteraction, Cache};
///
/// let command_interaction = CommandInteraction::new(); // Example setup
/// let anilist_cache: Arc<RwLock<Cache<String, String>>> = Arc::new(RwLock::new(Cache::new()));
///
/// let result = get_staff(&command_interaction, anilist_cache).await;
///
/// match result {
///     Ok(staff) => println!("Staff: {:?}", staff),
///     Err(err) => println!("Error: {:?}", err),
/// }
/// ```
///
/// [`CommandInteraction`]: crate::CommandInteraction
/// [`Cache`]: crate::Cache
/// [`Staff`]: crate::Staff
async fn get_staff(
	command_interaction: &CommandInteraction, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Staff> {
	let map = get_option_map_string(command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("staff_name"))
		.ok_or(anyhow!("No staff name specified"))?;

	let staff = if value.parse::<i32>().is_ok() {
		let var = StaffQuerryIdVariables {
			id: Some(value.parse()?),
		};

		let operation = StaffQuerryId::build(var);

		let data: GraphQlResponse<StaffQuerryId> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	} else {
		let var = StaffQuerrySearchVariables {
			search: Some(value),
		};

		let operation = StaffQuerrySearch::build(var);

		let data: GraphQlResponse<StaffQuerrySearch> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().staff.unwrap()
	};

	Ok(staff)
}

/// Generates a formatted string representation of a date, given an optional `FuzzyDate` object.
///
/// This function takes an `Option<FuzzyDate>` as input and converts it into a string representation.
/// If the `Option` is `None`, it returns an empty string. Otherwise, the components of the `FuzzyDate`
/// (day, month, and year) are combined into a string in the format `MM/DD/YYYY`, with optional components
/// included as available. For example:
///
/// - If only the month is provided, the output is `MM`.
/// - If both the month and day are provided, the output is `MM/DD`.
/// - If all three components (month, day, year) are provided, the output is `MM/DD/YYYY`.
///
/// # Parameters
/// - `option`: An optional `FuzzyDate` object containing the components of the date. Each component (`month`, `day`, `year`) is an `Option<u32>`.
///
/// # Returns
/// A `String` representing the formatted date. If `option` is `None`, returns an empty string. If components are missing, the format dynamically adjusts to exclude the missing parts.
///
/// # Examples
/// ```rust
/// let date = Some(FuzzyDate { month: Some(12), day: Some(25), year: Some(2023) });
/// assert_eq!(get_date(date), "12/25/2023");
///
/// let date = Some(FuzzyDate { month: Some(12), day: None, year: Some(2023) });
/// assert_eq!(get_date(date), "12/2023");
///
/// let date = Some(FuzzyDate { month: Some(12), day: None, year: None });
/// assert_eq!(get_date(date), "12");
///
/// let date = None;
/// assert_eq!(get_date(date), "");
/// ```
///
/// # Behavior
/// - If `month` is present, it is added to the string first.
/// - If `day` is present, it is appended to the string, separated from the previous part by a `/` if `month` exists.
/// - If `year` is present, it is appended to the string, separated from the previous part by a `/` if `day` exists.
///
/// # Edge Cases
/// - Returns an empty string if the input `option` is `None`.
/// - Handles combinations of missing date parts gracefully, providing the appropriate format based on the available components.
fn get_date(option: Option<FuzzyDate>) -> String {
	if option.is_none() {
		return String::new();
	}
	let date = option.unwrap();

	let mut date_string = String::new();

	let mut day = false;

	let mut month = false;

	if let Some(m) = date.month {
		month = true;

		date_string.push_str(m.to_string().as_str())
	}

	if let Some(d) = date.day {
		day = true;

		if month {
			date_string.push('/')
		}

		date_string.push_str(d.to_string().as_str())
	}

	if let Some(y) = date.year {
		if day {
			date_string.push('/')
		}

		date_string.push_str(y.to_string().as_str())
	}

	date_string
}

/// Returns a full name constructed by combining two optional string components.
///
/// This function takes two `Option<&str>` inputs, `a` and `b`. It produces an `Option<String>`
/// based on the following rules:
///
/// - If both `a` and `b` are `Some`, it concatenates them with a `/` separator and returns as `Some(String)`.
/// - If only `a` is `Some`, it returns `a` as `Some(String)`.
/// - If only `b` is `Some`, it returns `b` as `Some(String)`.
/// - If both `a` and `b` are `None`, it returns `None`.
///
/// # Arguments
///
/// * `a` - An optional string slice representing the first component of the name.
/// * `b` - An optional string slice representing the second component of the name.
///
/// # Returns
///
/// An `Option<String>` representing the combined full name or `None` if both inputs are `None`.
///
/// # Examples
///
/// ```
/// let full_name = get_full_name(Some("Alice"), Some("Smith"));
/// assert_eq!(full_name, Some("Alice/Smith".to_string()));
///
/// let first_name_only = get_full_name(Some("Alice"), None);
/// assert_eq!(first_name_only, Some("Alice".to_string()));
///
/// let last_name_only = get_full_name(None, Some("Smith"));
/// assert_eq!(last_name_only, Some("Smith".to_string()));
///
/// let no_name = get_full_name(None, None);
/// assert_eq!(no_name, None);
/// ```
fn get_full_name(a: Option<&str>, b: Option<&str>) -> Option<String> {
	match (a, b) {
		(Some(a), Some(b)) => Some(format!("{}/{}", a, b)),
		(Some(a), None) => Some(a.to_string()),
		(None, Some(b)) => Some(b.to_string()),
		(None, None) => None,
	}
}
