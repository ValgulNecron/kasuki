//! A module for handling the `LevelCommand` and calculating user level
//! progress based on anime and manga statistics. Additionally, this module
//! supports fetching user data from AniList and database, processing and
//! preparing data for the embed visualization.
use once_cell::sync::Lazy;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};

use crate::command::anilist_user::user::get_user;
use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::level::load_localization_level;
use crate::structure::run::anilist::user::{get_color, get_completed, get_user_url};
use crate::{get_url, impl_command};
use anyhow::anyhow;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use small_fixed_array::FixedString;

#[derive(Clone)]
pub struct LevelCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for LevelCommand,
	get_contents = |self_: LevelCommand| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();

	let anilist_cache = bot_data.anilist_cache.clone();
		let config = bot_data.config.clone();
		let map = get_option_map_string(&command_interaction);

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
		let db_connection = bot_data.db_connection.clone();

		// Load the localized level strings
		let level_localised = load_localization_level(guild_id, db_connection).await?;

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

		let next_level_display = if next_xp <= 1.0 {
			// Using 1.0 as a threshold assuming xp_for_next_level won't be fractional in practice.
			// This effectively means MAX_XP if next_xp is calculated to be 0 or less,
			// which would happen if current XP is already at or beyond the max level XP.
			level_localised.max.clone()
		} else {
			next_xp.to_string()
		};

		let mut embed_content = EmbedContent::new(user.clone().name)
			.description(
				level_localised
					.desc
					.replace("$username$", username.as_str())
					.replace("$level$", level.to_string().as_str())
					.replace("$xp$", format!("{:.2}", xp).as_str())
					.replace("$actual$", format!("{:.2}", actual).as_str())
					.replace("$next$", next_level_display.as_str()),
			)
			.thumbnail(user.clone().avatar.unwrap().large.clone().unwrap())
			.url(get_user_url(&user.id))
			.colour(get_color(user.clone()));

		// Add the user's banner image to the embed if it exists
		if let Some(banner_image) = &user.banner_image {
			embed_content = embed_content.images_url(banner_image.clone());
		}

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
);

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
/// - The XP required to reach the next level (`f64`)
fn generate_levels() -> [(u32, f64, f64); TOTAL_LEVELS] {
    let mut levels = [(0, 0.0, 0.0); TOTAL_LEVELS];
    for level in 0..(TOTAL_LEVELS as u32) {
        let required_xp = xp_required_for_level(level);
        let next_level_xp = if level < (TOTAL_LEVELS as u32 - 1) {
            xp_required_for_level(level + 1)
        } else {
            MAX_XP
        };
        levels[level as usize] = (level, required_xp, next_level_xp);
    }
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
/// - `u32`: The user's current level.
/// - `f64`: The amount of XP the user has accumulated within their current level.
/// - `f64`: The amount of XP required to reach the next level from the start of the current level.
fn get_level(xp: f64) -> (u32, f64, f64) {
    for &(level, required_xp, next_level_xp) in LEVELS.iter().rev() {
        if xp >= required_xp {
            let xp_in_current_level = xp - required_xp;
            let xp_for_next_level = next_level_xp - required_xp;
            // Ensure next_xp is not zero to prevent division by zero in description
            let xp_for_next_level = if xp_for_next_level <= 0.0 { 1.0 } else { xp_for_next_level };
            return (level, xp_in_current_level, xp_for_next_level);
        }
    }
    (0, 0.0, xp_required_for_level(1)) // Fallback: level 0, 0 XP in current, XP to level 1
}

/// Calculates the experience points (XP) required to reach a given level.
///
/// # Parameters
/// - `level`: The target level for which the required XP is being calculated.
///   Level must be a non-negative integer (`u32`).
///
/// # Returns
/// A `f64` representing the total experience points needed to reach the specified level.
fn xp_required_for_level(level: u32) -> f64 {
    const BASE_XP: f64 = 100.0;
    const GROWTH_RATE: f64 = 1.12;

    match level {
        0 => 0.0,
        1 => BASE_XP,
        2..=25 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64),
        26..=50 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 1.2,
        51..=75 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 1.5,
        76..=100 => BASE_XP * GROWTH_RATE.powf((level - 1) as f64) * 2.0,
        _ => xp_required_for_level(100) + (level - 100) as f64 * 10000.0, // Linear growth after level 100
    }
}
