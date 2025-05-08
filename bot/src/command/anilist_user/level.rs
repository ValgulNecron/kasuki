//! A module for handling the `LevelCommand` and calculating user level
//! progress based on anime and manga statistics. Additionally, this module
//! supports fetching user data from AniList and database, processing and
//! preparing data for the embed visualization.
use once_cell::sync::Lazy;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};

use crate::command::anilist_user::user::get_user;
use crate::command::command_trait::{Command, EmbedContent, EmbedType};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::level::load_localization_level;
use crate::structure::run::anilist::user::{get_color, get_completed, get_user_url};
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use small_fixed_array::FixedString;

/// Represents a level command within a Discord bot context.
///
/// The `LevelCommand` struct encapsulates the necessary context and interaction
/// data for processing a command related to leveling functionality in a Discord bot.
///
/// # Fields
///
/// * `ctx` - The bot's context (`SerenityContext`) which provides access to
///   various
pub struct LevelCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for LevelCommand {
	///
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the stored `CommandInteraction`.
	///
	/// This method provides access to the `CommandInteraction` instance
	/// associated with the current object. The returned reference allows
	/// the caller to inspect the command interaction but not modify it.
	///
	/// # Returns
	/// A reference
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	///
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let db_config = config.db.clone();
		let map = get_option_map_string(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("username"));

		let data = match user {
			Some(value) => get_user(value, anilist_cache).await?,
			None => {
				let user_id = &command_interaction.user.id.to_string();

				let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

				let row = RegisteredUser::find()
					.filter(Column::UserId.eq(user_id))
					.one(&connection)
					.await?;

				let user = row.ok_or(anyhow!(
					"No user specified or linked to this discord account",
				))?;

				get_user(user.anilist_id.to_string().as_str(), anilist_cache).await?
			},
		};

		let user = data;

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized level strings
		let level_localised = load_localization_level(guild_id, db_config).await?;

		// Clone the manga and anime statistics
		let statistics = user.statistics.clone().unwrap();

		let manga = statistics.manga.clone();

		let anime = statistics.anime.clone();

		// Calculate the number of manga and anime completed
		let manga_completed = if let Some(manga) = manga.clone() {
			get_completed(manga.statuses.unwrap())
		} else {
			0
		};

		let anime_completed = if let Some(anime) = anime.clone() {
			get_completed(anime.statuses.unwrap())
		} else {
			0
		};

		// Get the number of chapters read and minutes watched
		let chap_read = if let Some(manga) = manga.clone() {
			manga.chapters_read
		} else {
			0
		};

		let tw = if let Some(anime) = anime.clone() {
			anime.minutes_watched
		} else {
			0
		};

		// Calculate the experience points
		let xp = (8.0 * (manga_completed + anime_completed) as f64)
			+ (2.0 * chap_read as f64)
			+ (tw as f64 * 0.5);

		// Get the username
		let username = user.name.clone();

		// Calculate the level and progress
		let (level, actual, next_xp): (u32, f64, f64) = get_level(xp);

		let mut embed_content = EmbedContent::new(user.clone().name)
			.description(
				level_localised
					.desc
					.replace("$username$", username.as_str())
					.replace("$level$", level.to_string().as_str())
					.replace("$xp$", xp.to_string().as_str())
					.replace("$actual$", actual.to_string().as_str())
					.replace("$next$", next_xp.to_string().as_str()),
			)
			.thumbnail(Some(user.clone().avatar.unwrap().large.clone().unwrap()))
			.url(Some(get_user_url(&user.id)))
			.command_type(EmbedType::First)
			.colour(Some(get_color(user.clone())));

		// Add the user's banner image to the embed if it exists
		if let Some(banner_image) = &user.banner_image {
			embed_content = embed_content.images_url(Some(banner_image.clone()));
		}
		
		Ok(vec![embed_content])
	}
}

/// A constant that represents the maximum possible value for an XP (experience points) system,
pub const MAX_XP: f64 = f64::MAX;
///
pub const TOTAL_LEVELS: usize = 200;

/// A static variable `LEVELS` initialized lazily at runtime using the `generate_levels` function.
///
/// This static contains an array of tuples, where each tuple represents a level's details.
pub static LEVELS: Lazy<[(u32, f64, f64); TOTAL_LEVELS]> = Lazy::new(generate_levels);

/// Generates an array containing information about player levels and their respective XP requirements.
///
/// This function initializes an array where each element represents a level and contains three values:
/// - The level number (`u32`)
/// - The XP required to reach that level (`f64`)
/// - The XP required to reach the
fn generate_levels() -> [(u32, f64, f64); TOTAL_LEVELS] {
	let mut levels = [(0, 0.0, 0.0); TOTAL_LEVELS];
	for level in 0..=100 {
		let required_xp = xp_required_for_level(level);
		let next_level_xp = if level < 100 {
			xp_required_for_level(level + 1)
		} else {
			MAX_XP
		};
		levels[level as usize] = (level, required_xp, next_level_xp);
	}
	levels[101] = (101, MAX_XP, MAX_XP);
	levels
}

/// Determines the level of a user based on their experience points (XP)
/// and provides additional information about their progression within the level.
///
/// # Parameters
/// - `xp` (`f64`): The total experience points of the user.
///
/// # Returns
/// A tuple containing:
///
fn get_level(xp: f64) -> (u32, f64, f64) {
	for &(level, required_xp, next_level_xp) in LEVELS.iter().rev() {
		if xp >= required_xp {
			let xp_in_current_level = xp - required_xp;
			let xp_for_next_level = next_level_xp - required_xp;
			return (level, xp_in_current_level, xp_for_next_level);
		}
	}
	(0, 0.0, 100.0) // Default fallback
}

/// Calculates the experience points (XP) required to reach a given level.
///
/// # Parameters
/// - `level`: The target level for which the required XP is being calculated.
///   Level must be a non-negative integer (`u32`).
///
/// # Returns
/// A `f64
fn xp_required_for_level(level: u32) -> f64 {
	/// Represents the base experience points (XP) value used as a reference for calculation purposes in the system.
	///
	/// # Constant
	/// `BASE_XP` is a floating-point number initialized with a value of `100.0`.
	///
	/// # Usage
	/// This value serves as the foundational XP value that other calculations or adjustments
	/// (e
	const BASE_XP: f64 = 100.0;
	///
	const GROWTH_RATE: f64 = 1.12;

	match level {
		0 => 0.0,
		1 => BASE_XP,
		2..=25 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64),
		26..=50 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 1.2,
		51..=75 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 1.5,
		76..=100 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 2.0,
		_ => MAX_XP,
	}
}
