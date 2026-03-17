//! Represents a command to compare two AniList user profiles based on their anime and manga statistics.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use serenity::builder::{
	CreateComponent, CreateContainer, CreateContainerComponent, CreateSection,
	CreateSectionAccessory, CreateSectionComponent, CreateSeparator, CreateTextDisplay,
	CreateThumbnail, CreateUnfurledMediaItem,
};
use shared::localization::{LanguageIdentifier, USABLE_LOCALES};
use small_fixed_array::FixedString;
use tracing::trace;

use crate::command::context::CommandContext;
use crate::command::component_version::ComponentVersion::V2;
use crate::command::component_version::ComponentVersion2;
use crate::command::embed_content::EmbedsContents;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::user::{
	MediaListStatus, User, UserGenreStatistic, UserStatisticTypes, UserStatistics,
	UserStatistics2, UserStatusStatistic, UserTagStatistic, get_user,
};

#[slash_command(
	name = "compare", desc = "Compare 2 user.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [
		(name = "username", desc = "Username of the first user you want to compare.", arg_type = String, required = true, autocomplete = true),
		(name = "username2", desc = "Username of the second user you want to compare.", arg_type = String, required = true, autocomplete = true)
	],
)]
async fn compare_command(self_: CompareCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("username"))
		.cloned()
		.unwrap_or(String::new());

	let value2 = map
		.get(&FixedString::from_str_trunc("username2"))
		.cloned()
		.unwrap_or(String::new());

	// Fetch the user data for both users
	let user: User = get_user(&value, anilist_cache.clone()).await?;

	let user2: User = get_user(&value2, anilist_cache).await?;

	// Get the language identifier for localization
	let lang_id = cx.lang_id().await;

	// Clone the user data
	let username = user.name.clone();

	let username2 = user2.name.clone();
	// Calculate the affinity between the two users
	let affinity = get_affinity(
		user.statistics.clone().unwrap(),
		user2.statistics.clone().unwrap(),
	);

	let mut u1_text = String::new();
	let mut u2_text = String::new();
	let mut common_text = String::new();

	// Add the affinity to the description string
	{
		let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
		args.insert(Cow::Borrowed("var1"), FluentValue::from(username.as_str()));
		args.insert(Cow::Borrowed("var2"), FluentValue::from(username2.as_str()));
		args.insert(
			Cow::Borrowed("var3"),
			FluentValue::from(affinity.to_string()),
		);
		common_text.push_str(&USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"anilist_user_compare-affinity",
			&args,
		));
		common_text.push('\n');
	}

	let statistics = user.statistics.unwrap();

	let statistics2 = user2.statistics.unwrap();

	let anime = statistics.anime.unwrap();

	let anime2 = statistics2.anime.unwrap();

	let minutes_watched = anime.minutes_watched;

	let minutes_watched2 = anime2.minutes_watched;

	let count = anime.count;

	let count2 = anime2.count;

	// Compare the count of anime watched by the two users and add the result to the description string
	match count.cmp(&count2) {
		std::cmp::Ordering::Greater => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username2.as_str()),
			);
			u1_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_anime",
				&args,
			));
			u1_text.push('\n');
		},
		std::cmp::Ordering::Less => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username2.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username.as_str()),
			);
			u2_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_anime",
				&args,
			));
			u2_text.push('\n');
		},
		std::cmp::Ordering::Equal => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(Cow::Borrowed("var1"), FluentValue::from(username.as_str()));
			args.insert(Cow::Borrowed("var2"), FluentValue::from(username2.as_str()));
			common_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-same_anime",
				&args,
			));
			common_text.push('\n');
		},
	}

	// Compare the minutes watched by the two users and add the result to the description string
	match minutes_watched.cmp(&minutes_watched2) {
		std::cmp::Ordering::Greater => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username2.as_str()),
			);
			u1_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_watch_time",
				&args,
			));
			u1_text.push('\n');
		},
		std::cmp::Ordering::Less => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username2.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username.as_str()),
			);
			u2_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_watch_time",
				&args,
			));
			u2_text.push('\n');
		},
		std::cmp::Ordering::Equal => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(Cow::Borrowed("var1"), FluentValue::from(username.as_str()));
			args.insert(Cow::Borrowed("var2"), FluentValue::from(username2.as_str()));
			common_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-same_watch_time",
				&args,
			));
			common_text.push('\n');
		},
	}

	// Get the tags of the anime watched by the two users and add the comparison to the description string
	let tag = get_tag(&anime.tags.unwrap());

	let tag2 = get_tag(&anime2.tags.unwrap());

	common_text.push_str(&diff(
		&tag,
		&tag2,
		"anilist_user_compare-tag_anime",
		"anilist_user_compare-same_tag_anime",
		&username,
		&username2,
		&lang_id,
	));
	common_text.push('\n');

	// Get the genres of the anime watched by the two users and add the comparison to the description string
	let genre = get_genre(&anime.genres.unwrap());

	let genre2 = get_genre(&anime2.genres.unwrap());

	common_text.push_str(&diff(
		&genre,
		&genre2,
		"anilist_user_compare-genre_anime",
		"anilist_user_compare-same_genre_anime",
		&username,
		&username2,
		&lang_id,
	));
	common_text.push('\n');

	let manga = statistics.manga.unwrap();

	let manga2 = statistics2.manga.unwrap();

	let count = manga.count;

	let count2 = manga2.count;

	let chapters_read = manga.chapters_read;

	let chapters_read2 = manga2.chapters_read;

	// Compare the count of manga read by the two users and add the result to the description string
	match count.cmp(&count2) {
		std::cmp::Ordering::Greater => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username2.as_str()),
			);
			u1_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_manga",
				&args,
			));
			u1_text.push('\n');
		},
		std::cmp::Ordering::Less => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username2.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username.as_str()),
			);
			u2_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_manga",
				&args,
			));
			u2_text.push('\n');
		},
		std::cmp::Ordering::Equal => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(Cow::Borrowed("var1"), FluentValue::from(username.as_str()));
			args.insert(Cow::Borrowed("var2"), FluentValue::from(username2.as_str()));
			common_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-same_manga",
				&args,
			));
			common_text.push('\n');
		},
	}

	// Compare the chapters read by the two users and add the result to the description string
	match chapters_read.cmp(&chapters_read2) {
		std::cmp::Ordering::Greater => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username2.as_str()),
			);
			u1_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_manga_chapter",
				&args,
			));
			u1_text.push('\n');
		},
		std::cmp::Ordering::Less => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(
				Cow::Borrowed("greater"),
				FluentValue::from(username2.as_str()),
			);
			args.insert(
				Cow::Borrowed("lesser"),
				FluentValue::from(username.as_str()),
			);
			u2_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-more_manga_chapter",
				&args,
			));
			u2_text.push('\n');
		},
		std::cmp::Ordering::Equal => {
			let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
			args.insert(Cow::Borrowed("var1"), FluentValue::from(username.as_str()));
			args.insert(Cow::Borrowed("var2"), FluentValue::from(username2.as_str()));
			common_text.push_str(&USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"anilist_user_compare-same_manga_chapter",
				&args,
			));
			common_text.push('\n');
		},
	}

	// Get the tags of the manga read by the two users and add the comparison to the description string
	let tag = get_tag(&manga.tags.unwrap());

	let tag2 = get_tag(&manga2.tags.unwrap());

	common_text.push_str(&diff(
		&tag,
		&tag2,
		"anilist_user_compare-tag_manga",
		"anilist_user_compare-same_tag_manga",
		&username,
		&username2,
		&lang_id,
	));
	common_text.push('\n');

	// Get the genres of the manga read by the two users and add the comparison to the description string
	let genre = get_genre(&manga.genres.unwrap());

	let genre2 = get_genre(&manga2.genres.unwrap());

	common_text.push_str(&diff(
		&genre,
		&genre2,
		"anilist_user_compare-genre_manga",
		"anilist_user_compare-same_genre_manga",
		&username,
		&username2,
		&lang_id,
	));
	common_text.push('\n');

	let section_u1 = (
		vec![CreateSectionComponent::TextDisplay(CreateTextDisplay::new(
			u1_text,
		))],
		CreateSectionAccessory::Thumbnail(CreateThumbnail::new(CreateUnfurledMediaItem::new(
			user.avatar.unwrap().large.unwrap(),
		))),
	);
	let section_u2 = (
		vec![CreateSectionComponent::TextDisplay(CreateTextDisplay::new(
			u2_text,
		))],
		CreateSectionAccessory::Thumbnail(CreateThumbnail::new(CreateUnfurledMediaItem::new(
			user2.avatar.unwrap().large.unwrap(),
		))),
	);
	let common = CreateContainerComponent::TextDisplay(CreateTextDisplay::new(common_text));

	let u1 = CreateContainerComponent::Section(CreateSection::new(section_u1.0, section_u1.1));
	let u2 = CreateContainerComponent::Section(CreateSection::new(section_u2.0, section_u2.1));

	let data = vec![
		common,
		CreateContainerComponent::Separator(CreateSeparator::new()),
		u1,
		CreateContainerComponent::Separator(CreateSeparator::new()),
		u2,
	];

	let create_components = CreateComponent::Container(CreateContainer::new(data));

	let embed_contents =
		EmbedsContents::new(vec![]).action_row(V2(ComponentVersion2 {
			components: Cow::Owned(vec![create_components]),
		}));

	Ok(embed_contents)
}

/// Calculates the affinity score between two users based on their anime and manga preferences.
///
/// This function computes a similarity score (affinity) between two users by comparing their watched anime
/// and read manga statistics. The score is based on the Jaccard index of tags and genres, combined with
/// additional affinity measures for anime and manga preferences.
///
/// # Parameters
/// - `s1` (`UserStatisticTypes`): A data structure containing anime and manga statistics for the first user.
/// - `s2` (`UserStatisticTypes`): A data structure containing anime and manga statistics for the second user.
///
/// # Returns
/// - `f64`: The affinity score between the two users as a percentage.
///
/// # Process
/// 1. Extracts the anime and manga data (`tags` and `genres`) for both users.
/// 2. Calculates the affinity based on the Jaccard index for the tags of the users' watched anime and read manga.
/// 3. Adds the Jaccard index calculated for genres of watched anime and read manga.
/// 4. Integrates an additional affinity score for anime (`other_affinity_anime`) and manga (`other_affinity_manga`).
/// 5. Combines the calculated values, averages certain measures, and multiplies the result by 100 to return a percentage.
///
/// # Notes
/// - If any of the anime or manga data for either user is `None`, the function will panic due to the use of `unwrap()`.
/// - This implementation assumes the existence of helper functions:
///   - `jaccard_index`
///   - `tag_string`
///   - `genre_string`
///   - `other_affinity_anime`
///   - `other_affinity_manga`
///
/// # Example
/// ```rust
/// let user1_stats = UserStatisticTypes {
///     anime: Some(anime_data1),
///     manga: Some(manga_data1),
/// };
/// let user2_stats = UserStatisticTypes {
///     anime: Some(anime_data2),
///     manga: Some(manga_data2),
/// };
///
/// let affinity_score = get_affinity(user1_stats, user2_stats);
/// println!("Affinity score: {:.2}%", affinity_score);
/// ```
///
/// # Panics
/// - The function panics if any of the anime or manga statistics (`tags` or `genres`) are `None`.
/// - Ensure that `.unwrap()` usage handles non-empty values or refactor to handle errors gracefully.
fn get_affinity(s1: UserStatisticTypes, s2: UserStatisticTypes) -> f64 {
	// Initialize the affinity
	let mut affinity: f64;

	let anime = s1.anime.clone().unwrap();

	let anime2 = s2.anime.clone().unwrap();

	// Calculate the Jaccard index of the tags of the anime watched by the users
	affinity = jaccard_index(
		&tag_string(&anime.tags.clone().unwrap()),
		&tag_string(&anime2.tags.clone().unwrap()),
	);

	// Add the Jaccard index of the genres of the anime watched by the users to the affinity
	affinity += jaccard_index(
		&genre_string(&anime.genres.clone().unwrap()),
		&genre_string(&anime2.genres.clone().unwrap()),
	);

	let manga = s1.manga.clone().unwrap();

	let manga2 = s2.manga.clone().unwrap();

	let mut affinity2 = jaccard_index(
		&tag_string(&manga.tags.clone().unwrap()),
		&tag_string(&manga2.tags.clone().unwrap()),
	);

	affinity2 += jaccard_index(
		&genre_string(&manga.genres.clone().unwrap()),
		&genre_string(&manga2.genres.clone().unwrap()),
	);

	// Calculate the affinity between the anime watched by the users
	let mut affinity3 = other_affinity_anime(anime, anime2);

	// Add the affinity between the manga read by the users to the affinity
	affinity3 += other_affinity_manga(manga, manga2);

	// Return the total affinity divided by 2 and multiplied by 100
	((affinity / 2.0) + (affinity2 / 2.0) + affinity3) * 100.0
}

/// Calculates the affinity between two users' anime-watching statistics
/// based on various factors, such as anime status categories, count, minutes
/// watched, and score statistics.
///
/// # Parameters
/// - `anime`: A `UserStatistics` struct containing information about the
///   first user's anime-watching habits.
/// - `anime0`: A `UserStatistics` struct containing information about the
///   second user's anime-watching habits.
///
/// # Returns
/// - A `f64` value representing the affinity score between two users, scaled
///   from 0.0 to 1.0. The higher the value, the more similar the two users are
///   in terms of their anime-watching statistics.
///
/// # Calculation
/// 1. Fetch the number of anime in each status category (e.g., current, planning,
///    completed, dropped, paused, repeating) for both users using the
///    `get_number_by_status` function.
/// 2. Increase the affinity score for each matching count in the status categories
///    between the two users.
/// 3. Compare overall statistics for both users (e.g., total anime count, minutes
///    watched, standard deviation of scores, and mean score). Increase the affinity
///    score for each matching statistic.
/// 4. Divide the total affinity score by 20.0 to normalize the value.
///
/// # Example
/// ```rust
/// let user1_stats = UserStatistics { statuses: Some(statuses1), count: 100, minutes_watched: 30000, standard_deviation: 1.5, mean_score: 8.0 };
/// let user2_stats = UserStatistics { statuses: Some(statuses2), count: 100, minutes_watched: 30000, standard_deviation: 1.5, mean_score: 8.0 };
/// let affinity = other_affinity_anime(user1_stats, user2_stats);
/// println!("Affinity: {}", affinity);
/// ```
///
/// # Notes
/// - The `get_number_by_status` function is assumed to return a tuple containing the
///   counts for each anime status category (e.g., current, planning, etc.).
/// - The affinity score is a fraction, where higher similarity in status counts and
///   statistics results in a higher score.
fn other_affinity_anime(anime: UserStatistics, anime0: UserStatistics) -> f64 {
	// Retrieve the number of anime in each status category for both anime
	let (current, planning, completed, dropped, paused, repeating) =
		get_number_by_status(anime.statuses.unwrap());

	let (current0, planning0, completed0, dropped0, paused0, repeating0) =
		get_number_by_status(anime0.statuses.unwrap());

	// Initialize the affinity to 0
	let mut affinity = 0.0;

	// Increase the affinity by 1 for each matching status category
	if current == current0 {
		affinity += 1f64
	}

	if planning == planning0 {
		affinity += 1f64
	}

	if completed == completed0 {
		affinity += 1f64
	}

	if dropped == dropped0 {
		affinity += 1f64
	}

	if paused == paused0 {
		affinity += 1f64
	}

	if repeating == repeating0 {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the count of the anime is the same
	if anime.count == anime0.count {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the minutes watched is the same
	if anime.minutes_watched == anime0.minutes_watched {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the standard deviation of the scores is the same
	if anime.standard_deviation == anime0.standard_deviation {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the mean score is the same
	if anime.mean_score == anime0.mean_score {
		affinity += 1f64
	}

	// Return the total affinity divided by 20
	affinity / 20.0
}

/// Calculates the affinity score between two sets of user manga statistics.
///
/// The affinity score is determined by comparing various fields from two `UserStatistics2` objects.
/// The score increases by a fixed amount for each matching attribute or status category, and the final
/// score is normalized by dividing the total affinity by 20.
///
/// # Parameters
/// - `manga`: The first set of user manga statistics (`UserStatistics2`).
/// - `manga0`: The second set of user manga statistics (`UserStatistics2`).
///
/// # Returns
/// A `f64` value representing the normalized affinity score between the two users, where:
/// - A higher score indicates greater similarity in manga affinities.
/// - The maximum possible score is 1.0.
///
/// # Calculation
/// 1. Compares the number of manga in each status category (e.g., current, planning, completed,
///    dropped, paused, repeating) using `get_number_by_status`.
/// 2. Increments the affinity score by 1 for each matching status category.
/// 3. Increments the affinity score by 1 for each of the following matching attributes:
///    - Total count of manga.
///    - Number of chapters read.
///    - Standard deviation of scores.
///    - Mean score.
/// 4. Normalizes the score by dividing the total score by 20.
///
/// # Assumptions
/// - The `statuses` field in `UserStatistics2` must not be `None`.
/// - The function assumes that `get_number_by_status` correctly splits the statuses into
///   their respective categories (current, planning, completed, etc.).
///
/// # Example
/// ```
/// let manga1 = UserStatistics2 { /* fields populated */ };
/// let manga2 = UserStatistics2 { /* fields populated */ };
/// let affinity = other_affinity_manga(manga1, manga2);
/// println!("Affinity Score: {}", affinity);
/// ```
///
/// # Notes
/// - The function assumes that `UserStatistics2` contains comparable fields such as `count`,
///   `chapters_read`, `standard_deviation`, and `mean_score`.
/// - To broaden the affinity metric, weights or additional factors can be introduced.
fn other_affinity_manga(manga: UserStatistics2, manga0: UserStatistics2) -> f64 {
	// Retrieve the number of manga in each status category for both manga
	let (current, planning, completed, dropped, paused, repeating) =
		get_number_by_status(manga.statuses.unwrap());

	let (current0, planning0, completed0, dropped0, paused0, repeating0) =
		get_number_by_status(manga0.statuses.unwrap());

	// Initialize the affinity to 0
	let mut affinity = 0.0;

	// Increase the affinity by 1 for each matching status category
	if current == current0 {
		affinity += 1f64
	}

	if planning == planning0 {
		affinity += 1f64
	}

	if completed == completed0 {
		affinity += 1f64
	}

	if dropped == dropped0 {
		affinity += 1f64
	}

	if paused == paused0 {
		affinity += 1f64
	}

	if repeating == repeating0 {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the count of the manga is the same
	if manga.count == manga0.count {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the number of chapters read is the same
	if manga.chapters_read == manga0.chapters_read {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the standard deviation of the scores is the same
	if manga.standard_deviation == manga0.standard_deviation {
		affinity += 1f64
	}

	// Increase the affinity by 1 if the mean score is the same
	if manga.mean_score == manga0.mean_score {
		affinity += 1f64
	}

	// Return the total affinity divided by 20
	affinity / 20.0
}

/// Computes the Jaccard Index, which measures the similarity between two sets of strings.
///
/// The Jaccard Index is defined as the size of the intersection divided by the size of the union
/// of the two sets. It ranges from 0.0 (no overlap) to 1.0 (complete overlap).
///
/// # Arguments
///
/// * `a` - A reference to a slice of `String` representing the first set.
/// * `b` - A reference to a slice of `String` representing the second set.
///
/// # Returns
///
/// A `f64` value representing the Jaccard Index, which quantifies the similarity between the two sets.
///
/// # Example
///
/// ```
/// use std::collections::HashSet;
///
/// let set_a = vec![String::from("apple"), String::from("banana")];
/// let set_b = vec![String::from("banana"), String::from("cherry")];
///
/// let jaccard = jaccard_index(&set_a, &set_b);
/// assert_eq!(jaccard, 1.0 / 3.0); // "banana" is in the intersection, total unique items are 3
/// ```
///
/// # Complexity
///
/// The function has a time complexity of approximately O(n + m), where `n` and `m` are the sizes
/// of `a` and `b` respectively, as it builds hash sets and computes the intersection and union.
///
/// # Notes
///
/// - Duplicates in the input arrays do not affect the result, as the comparison operates on sets.
fn jaccard_index(a: &[String], b: &[String]) -> f64 {
	let set_a: HashSet<_> = a.iter().collect();

	let set_b: HashSet<_> = b.iter().collect();

	let intersection = set_a.intersection(&set_b).count();

	let union = set_a.union(&set_b).count();

	intersection as f64 / union as f64
}

/// Generates a tuple containing the count of users associated with various statuses from a vector of `UserStatusStatistic` options.
///
/// # Arguments
///
/// * `s` - A `Vec` containing optional `UserStatusStatistic` values that hold the count and status data.
///
/// # Returns
///
/// A tuple of six integers `(current, planning, completed, dropped, paused, repeating)` where:
/// - `current` refers to the count of users with the "CURRENT" status.
/// - `planning` refers to the count of users with the "PLANNING" status.
/// - `completed` refers to the count of users with the "COMPLETED" status.
/// - `dropped` refers to the count of users with the "DROPPED" status.
/// - `paused` refers to the count of users with the "PAUSED" status.
/// - `repeating` refers to the count of users with the "REPEATING" status.
///
/// If a status is not present in the input vector or is invalid, the corresponding count in the tuple will default to 0.
///
/// # Panics
///
/// This function will panic if any `Option<UserStatusStatistic>` in the input vector is `None`, or if the `status` field of
/// `UserStatusStatistic` is `None`. Ensure the input data is valid and properly populated.
///
/// # Example
///
/// ```rust
/// let statuses = vec![
///     Some(UserStatusStatistic { status: Some("CURRENT".to_string()), count: 10 }),
///     Some(UserStatusStatistic { status: Some("PLANNING".to_string()), count: 5 }),
///     Some(UserStatusStatistic { status: Some("COMPLETED".to_string()), count: 15 }),
/// ];
///
/// let result = get_number_by_status(statuses);
/// assert_eq!(result, (10, 5, 15, 0, 0, 0)); // "DROPPED", "PAUSED", and "REPEATING" statuses default to 0
/// ```
fn get_number_by_status(s: Vec<Option<UserStatusStatistic>>) -> (i32, i32, i32, i32, i32, i32) {
	let mut current = 0;

	let mut planning = 0;

	let mut completed = 0;

	let mut dropped = 0;

	let mut paused = 0;

	let mut repeating = 0;

	for statuses in s {
		let statuses = statuses.unwrap();

		let status = statuses.status.unwrap();

		match status {
			MediaListStatus::Current => current = statuses.count,
			MediaListStatus::Planning => planning = statuses.count,
			MediaListStatus::Completed => completed = statuses.count,
			MediaListStatus::Dropped => dropped = statuses.count,
			MediaListStatus::Paused => paused = statuses.count,
			MediaListStatus::Repeating => repeating = statuses.count,
		}
	}

	(current, planning, completed, dropped, paused, repeating)
}

/// Converts a slice of `Option<UserTagStatistic>` into a `Vec<String>`
/// by extracting the name of the tag from each element.
///
/// # Arguments
/// * `vec` - A slice of `Option<UserTagStatistic>`. Each element in the slice
///   is expected to be either `None` or `Some(UserTagStatistic)` containing a nested tag name.
///
/// # Returns
/// A `Vec<String>` containing the tag names extracted from the input slice.
///
/// # Panics
/// This function will panic if:
/// - An element in the slice is `None` (i.e., unwrap on `Option` fails).
/// - The `tag` field in `UserTagStatistic` is `None`.
///
/// # Example
/// ```rust
/// #[derive(Clone)]
/// struct Tag {
///     name: String,
/// }
///
/// #[derive(Clone)]
/// struct UserTagStatistic {
///     tag: Option<Tag>,
/// }
///
/// let user_tags = vec![
///     Some(UserTagStatistic {
///         tag: Some(Tag { name: "tag1".to_string() }),
///     }),
///     Some(UserTagStatistic {
///         tag: Some(Tag { name: "tag2".to_string() }),
///     }),
/// ];
///
/// let result = tag_string(&user_tags);
/// assert_eq!(result, vec!["tag1".to_string(), "tag2".to_string()]);
/// ```
fn tag_string(vec: &[Option<UserTagStatistic>]) -> Vec<String> {
	vec.iter()
		.map(|tag| {
			let tag = tag.clone().unwrap();

			tag.tag.unwrap().name.clone()
		})
		.collect()
}

/// Converts a slice of `Option<UserGenreStatistic>` into a `Vec<String>` containing the unwrapped genre strings.
///
/// # Arguments
///
/// * `vec` - A slice of `Option<UserGenreStatistic>` objects. Each element of the slice is expected to be of type `Option`
///   wrapping the `UserGenreStatistic` struct.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the genre strings extracted and unwrapped from the provided `vec`.
///
/// # Panics
///
/// This function will panic if:
/// 1. Any `Option<UserGenreStatistic>` in the input slice is `None`.
/// 2. The `genre` field inside any unwrapped `UserGenreStatistic` is `None`.
///
/// Ensure that all elements in the slice are properly populated with values before calling this function.
///
/// # Example
///
/// ```rust
/// #[derive(Clone)]
/// struct UserGenreStatistic {
///     genre: Option<String>,
/// }
///
/// let user_genres = vec![
///     Some(UserGenreStatistic { genre: Some("Rock".to_string()) }),
///     Some(UserGenreStatistic { genre: Some("Jazz".to_string()) }),
///     Some(UserGenreStatistic { genre: Some("Pop".to_string()) }),
/// ];
///
/// let genres = genre_string(&user_genres);
/// assert_eq!(genres, vec!["Rock".to_string(), "Jazz".to_string(), "Pop".to_string()]);
/// ```
fn genre_string(vec: &[Option<UserGenreStatistic>]) -> Vec<String> {
	vec.iter()
		.filter_map(|genre| genre.as_ref()?.genre.clone())
		.collect()
}

/// Retrieves the name of the tag from a slice of optional `UserTagStatistic` values.
///
/// # Arguments
///
/// * `tags` - A slice of `Option<UserTagStatistic>` representing a collection of user tag statistics.
///
/// # Returns
///
/// * A `String` containing the name of the tag if the first element in the slice is `Some` and contains a valid tag name.
/// * Returns an empty `String` if the slice contains one or fewer elements or if the required data is not present.
///
/// # Panics
///
/// This function will panic if the first element of the `tags` slice is `None` or if the `tag` field or `name` field
/// of the extracted `UserTagStatistic` struct is `None`.
///
/// # Examples
///
/// ```
/// let tags = vec![
///     Some(UserTagStatistic { tag: Some(Tag { name: "example".to_string() }) }),
///     None,
/// ];
/// assert_eq!(get_tag(&tags), "example".to_string());
///
/// let empty_tags: Vec<Option<UserTagStatistic>> = vec![];
/// assert_eq!(get_tag(&empty_tags), "".to_string());
/// ```
fn get_tag(tags: &[Option<UserTagStatistic>]) -> String {
	if tags.len() > 1 {
		tags[0]
			.as_ref()
			.and_then(|t| Some(t.tag.as_ref()?.name.clone()))
			.unwrap_or_default()
	} else {
		String::new()
	}
}

/// Retrieves the genre from a list of optional user genre statistics.
///
/// # Arguments
///
/// * `genres` - A slice of `Option<UserGenreStatistic>`, where each element may contain
///   a user's genre statistic or be `None`.
///
/// # Returns
///
/// Returns a `String` representing the genre:
/// - If there is more than one item in the `genres` slice, the function attempts to retrieve
///   the genre from the first element. If the first element is `None` or its genre field is `None`,
///   it returns an empty string.
/// - If the slice has zero or one element, it returns an empty string.
///
/// # Panics
///
/// This function will panic if the `genres` slice has a length greater than 1 and the first
/// element is `None` (since `unwrap()` is called on `None`) or if the first element does
/// not have a genre.
///
/// # Examples
///
/// ```
/// let genres = vec![Some(UserGenreStatistic { genre: Some("Rock".to_string()) })];
/// assert_eq!(get_genre(&genres), "Rock");
///
/// let genres: Vec<Option<UserGenreStatistic>> = vec![];
/// assert_eq!(get_genre(&genres), "");
///
/// let genres = vec![None];
/// assert_eq!(get_genre(&genres), "");
/// ```
///
/// # Notes
///
/// - It is recommended to handle the case where the first element is `None` gracefully
///   instead of panicking.
/// - The use of `unwrap()` introduces potential runtime panics if the value is not properly checked.
///
/// ```rust
/// #[derive(Clone)]
/// struct UserGenreStatistic {
///     genre: Option<String>,
/// }
/// ```
fn get_genre(genres: &[Option<UserGenreStatistic>]) -> String {
	if genres.len() > 1 {
		genres[0]
			.as_ref()
			.and_then(|g| g.genre.clone())
			.unwrap_or_default()
	} else {
		String::new()
	}
}

/// Compares two strings for differences and generates a message based on the comparison.
///
/// # Parameters
/// - `a1`: A reference to the first string to compare.
/// - `a2`: A reference to the second string to compare.
/// - `diff_key`: The Fluent key to use if the strings are different.
/// - `same_key`: The Fluent key to use if the strings are the same.
/// - `username`: The name of the first user tied to `a1`.
/// - `username2`: The name of the second user tied to `a2`.
/// - `lang_id`: The language identifier for localization.
///
/// # Returns
/// A `String` containing the formatted message based on whether the strings were identical or different.
fn diff(
	a1: &str, a2: &str, diff_key: &str, same_key: &str, username: &str, username2: &str,
	lang_id: &LanguageIdentifier,
) -> String {
	let is_diff = a1 != a2;

	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(Cow::Borrowed("var1"), FluentValue::from(username));
	args.insert(Cow::Borrowed("var2"), FluentValue::from(username2));
	args.insert(Cow::Borrowed("var1a"), FluentValue::from(a1));
	args.insert(Cow::Borrowed("var2a"), FluentValue::from(a2));

	let info = if is_diff {
		USABLE_LOCALES.lookup_with_args(lang_id, diff_key, &args)
	} else {
		USABLE_LOCALES.lookup_with_args(lang_id, same_key, &args)
	};

	trace!(info);

	info
}
