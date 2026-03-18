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

use crate::command::component_version::ComponentVersion::V2;
use crate::command::component_version::ComponentVersion2;
use crate::command::context::CommandContext;
use crate::command::embed_content::EmbedsContents;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::user::{
	get_user, MediaListStatus, User, UserGenreStatistic, UserStatisticTypes, UserStatistics,
	UserStatistics2, UserStatusStatistic, UserTagStatistic,
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

	let user: User = get_user(&value, anilist_cache.clone()).await?;

	let user2: User = get_user(&value2, anilist_cache).await?;

	let lang_id = cx.lang_id().await;

	let username = user.name.clone();

	let username2 = user2.name.clone();
	let statistics = user.statistics.unwrap();
	let statistics2 = user2.statistics.unwrap();

	let affinity = get_affinity(&statistics, &statistics2);

	let mut u1_text = String::new();
	let mut u2_text = String::new();
	let mut common_text = String::new();

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

	let anime = statistics.anime.unwrap();

	let anime2 = statistics2.anime.unwrap();

	let minutes_watched = anime.minutes_watched;

	let minutes_watched2 = anime2.minutes_watched;

	let count = anime.count;

	let count2 = anime2.count;

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

	let embed_contents = EmbedsContents::new(vec![]).action_row(V2(ComponentVersion2 {
		components: Cow::Owned(vec![create_components]),
	}));

	Ok(embed_contents)
}

fn get_affinity(s1: &UserStatisticTypes, s2: &UserStatisticTypes) -> f64 {
	let anime = s1.anime.as_ref().unwrap();
	let anime2 = s2.anime.as_ref().unwrap();

	let mut affinity = jaccard_index(
		&tag_string(anime.tags.as_deref().unwrap()),
		&tag_string(anime2.tags.as_deref().unwrap()),
	);

	affinity += jaccard_index(
		&genre_string(anime.genres.as_deref().unwrap()),
		&genre_string(anime2.genres.as_deref().unwrap()),
	);

	let manga = s1.manga.as_ref().unwrap();
	let manga2 = s2.manga.as_ref().unwrap();

	let mut affinity2 = jaccard_index(
		&tag_string(manga.tags.as_deref().unwrap()),
		&tag_string(manga2.tags.as_deref().unwrap()),
	);

	affinity2 += jaccard_index(
		&genre_string(manga.genres.as_deref().unwrap()),
		&genre_string(manga2.genres.as_deref().unwrap()),
	);

	let mut affinity3 = other_affinity_anime(anime, anime2);
	affinity3 += other_affinity_manga(manga, manga2);

	((affinity / 2.0) + (affinity2 / 2.0) + affinity3) * 100.0
}

fn other_affinity_anime(anime: &UserStatistics, anime0: &UserStatistics) -> f64 {
	let (current, planning, completed, dropped, paused, repeating) =
		get_number_by_status(anime.statuses.as_deref().unwrap());

	let (current0, planning0, completed0, dropped0, paused0, repeating0) =
		get_number_by_status(anime0.statuses.as_deref().unwrap());

	let mut affinity = 0.0;

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
	if anime.count == anime0.count {
		affinity += 1f64
	}
	if anime.minutes_watched == anime0.minutes_watched {
		affinity += 1f64
	}
	if anime.standard_deviation == anime0.standard_deviation {
		affinity += 1f64
	}
	if anime.mean_score == anime0.mean_score {
		affinity += 1f64
	}

	affinity / 20.0
}

fn other_affinity_manga(manga: &UserStatistics2, manga0: &UserStatistics2) -> f64 {
	let (current, planning, completed, dropped, paused, repeating) =
		get_number_by_status(manga.statuses.as_deref().unwrap());

	let (current0, planning0, completed0, dropped0, paused0, repeating0) =
		get_number_by_status(manga0.statuses.as_deref().unwrap());

	let mut affinity = 0.0;

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
	if manga.count == manga0.count {
		affinity += 1f64
	}
	if manga.chapters_read == manga0.chapters_read {
		affinity += 1f64
	}
	if manga.standard_deviation == manga0.standard_deviation {
		affinity += 1f64
	}
	if manga.mean_score == manga0.mean_score {
		affinity += 1f64
	}

	affinity / 20.0
}

fn jaccard_index(a: &[String], b: &[String]) -> f64 {
	let set_a: HashSet<_> = a.iter().collect();

	let set_b: HashSet<_> = b.iter().collect();

	let intersection = set_a.intersection(&set_b).count();

	let union = set_a.union(&set_b).count();

	intersection as f64 / union as f64
}

fn get_number_by_status(s: &[Option<UserStatusStatistic>]) -> (i32, i32, i32, i32, i32, i32) {
	let mut current = 0;

	let mut planning = 0;

	let mut completed = 0;

	let mut dropped = 0;

	let mut paused = 0;

	let mut repeating = 0;

	for statuses in s {
		let statuses = statuses.as_ref().unwrap();

		let status = statuses.status.as_ref().unwrap();

		match *status {
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

fn tag_string(vec: &[Option<UserTagStatistic>]) -> Vec<String> {
	vec.iter()
		.filter_map(|tag| tag.as_ref()?.tag.as_ref().map(|t| t.name.clone()))
		.collect()
}

fn genre_string(vec: &[Option<UserGenreStatistic>]) -> Vec<String> {
	vec.iter()
		.filter_map(|genre| genre.as_ref()?.genre.clone())
		.collect()
}

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
