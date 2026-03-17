pub use shared::anilist::user::*;

use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use shared::localization::{LanguageIdentifier, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

pub async fn user_content<'a>(
	user: User, lang_id: &LanguageIdentifier,
) -> Result<EmbedsContents<'a>> {
	let mut field = Vec::new();

	let statistics = user
		.statistics
		.clone()
		.ok_or(anyhow!("Could not get the statistics"))?;

	let manga = statistics.manga.clone();
	let anime = statistics.anime.clone();

	if let Some(m) = &manga {
		if m.count > 0 {
			field.push(get_manga_field(user.id, lang_id, m.clone()))
		}
	}

	if let Some(a) = &anime {
		if a.count > 0 {
			field.push(get_anime_field(user.id, lang_id, a.clone()))
		}
	}

	let mut embed_content = EmbedContent::new(user.name.clone())
		.url(get_user_url(&user.id))
		.colour(get_color(user.clone()))
		.fields(field)
		.images_url(get_banner(&user.id));

	if let Some(avatar) = user.avatar {
		if let Some(large) = avatar.large {
			embed_content.thumbnail = Some(large)
		}
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

fn get_user_manga_url(user_id: i32) -> String {
	format!("https://anilist.co/user/{}/mangalist", user_id)
}

fn get_user_anime_url(user_id: i32) -> String {
	format!("https://anilist.co/user/{}/animelist", user_id)
}

fn get_manga_field(
	user_id: i32, lang_id: &LanguageIdentifier, manga: UserStatistics2,
) -> (String, String, bool) {
	(
		String::new(),
		get_manga_desc(manga, lang_id, user_id),
		false,
	)
}

fn get_anime_field(
	user_id: i32, lang_id: &LanguageIdentifier, anime: UserStatistics,
) -> (String, String, bool) {
	(
		String::new(),
		get_anime_desc(anime, lang_id, user_id),
		false,
	)
}

fn get_manga_desc(manga: UserStatistics2, lang_id: &LanguageIdentifier, user_id: i32) -> String {
	let mut desc = String::new();
	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("url"),
		FluentValue::from(get_user_manga_url(user_id)),
	);
	desc.push_str(
		USABLE_LOCALES
			.lookup_with_args(lang_id, "anilist_user_user-manga-title", &args)
			.as_str(),
	);
	desc = desc.replace("\u{2069}", "");
	desc = desc.replace("\u{2068}", "");
	desc.push('\n');

	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("count"),
		FluentValue::from(manga.count.to_string()),
	);
	args.insert(
		Cow::Borrowed("complete"),
		FluentValue::from(get_completed(manga.statuses.unwrap().clone()).to_string()),
	);
	args.insert(
		Cow::Borrowed("chap"),
		FluentValue::from(manga.chapters_read.to_string()),
	);
	args.insert(
		Cow::Borrowed("score"),
		FluentValue::from(manga.mean_score.to_string()),
	);
	args.insert(
		Cow::Borrowed("sd"),
		FluentValue::from(manga.standard_deviation.to_string()),
	);
	args.insert(
		Cow::Borrowed("tag_list"),
		FluentValue::from(get_tag_list(manga.tags.as_deref().unwrap_or_default())),
	);
	args.insert(
		Cow::Borrowed("genre_list"),
		FluentValue::from(get_genre_list(manga.genres.as_deref().unwrap_or_default())),
	);

	desc.push_str(
		USABLE_LOCALES
			.lookup_with_args(lang_id, "anilist_user_user-manga", &args)
			.as_str(),
	);
	desc
}

fn get_anime_desc(anime: UserStatistics, lang_id: &LanguageIdentifier, user_id: i32) -> String {
	let mut desc = String::new();
	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("url"),
		FluentValue::from(get_user_anime_url(user_id)),
	);

	desc.push_str(
		USABLE_LOCALES
			.lookup_with_args(lang_id, "anilist_user_user-anime-title", &args)
			.as_str(),
	);
	desc = desc.replace("\u{2069}", "");
	desc = desc.replace("\u{2068}", "");
	desc.push('\n');

	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("count"),
		FluentValue::from(anime.count.to_string()),
	);
	args.insert(
		Cow::Borrowed("complete"),
		FluentValue::from(get_completed(anime.statuses.clone().unwrap()).to_string()),
	);
	args.insert(
		Cow::Borrowed("duration"),
		FluentValue::from(get_anime_time_watch(anime.minutes_watched, lang_id)),
	);
	args.insert(
		Cow::Borrowed("score"),
		FluentValue::from(anime.mean_score.to_string()),
	);
	args.insert(
		Cow::Borrowed("sd"),
		FluentValue::from(anime.standard_deviation.to_string()),
	);
	args.insert(
		Cow::Borrowed("tag_list"),
		FluentValue::from(get_tag_list(anime.tags.as_deref().unwrap_or_default())),
	);
	args.insert(
		Cow::Borrowed("genre_list"),
		FluentValue::from(get_genre_list(anime.genres.as_deref().unwrap_or_default())),
	);
	desc.push_str(
		USABLE_LOCALES
			.lookup_with_args(lang_id, "anilist_user_user-anime", &args)
			.as_str(),
	);
	desc
}

fn get_anime_time_watch(i: i32, lang_id: &LanguageIdentifier) -> String {
	let mut min = i;
	let mut hour = 0;
	let mut days = 0;
	let mut week = 0;

	if min >= 60 {
		hour = min / 60;
		min %= 60;
	}

	if hour >= 24 {
		days = hour / 24;
		hour %= 24;
	}

	if days >= 7 {
		week = days / 7;
		days %= 7;
	}

	let mut tw = String::new();

	if week >= 1 {
		let week_label = match week {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-week"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-weeks"),
		};
		tw.push_str(&format!("{}{}", week_label, week));
	}

	if days >= 1 {
		let day_label = match days {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-day"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-days"),
		};
		tw.push_str(&format!("{}{}", day_label, days));
	}

	if hour >= 1 {
		let hour_label = match hour {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-hour"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-hours"),
		};
		tw.push_str(&format!("{}{}", hour_label, hour));
	}

	if min >= 1 {
		let min_label = match min {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-minute"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-minutes"),
		};
		tw.push_str(&format!("{}{}", min_label, min));
	}
	tw
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_user(profile_color: Option<&str>) -> User {
		User {
			id: 1,
			name: "TestUser".to_string(),
			avatar: None,
			statistics: None,
			options: Some(UserOptions {
				profile_color: profile_color.map(|s| s.to_string()),
			}),
			banner_image: None,
		}
	}

	fn make_statuses(statuses: Vec<(&str, i32)>) -> Vec<Option<UserStatusStatistic>> {
		statuses
			.into_iter()
			.map(|(status, count)| {
				let s = match status {
					"COMPLETED" => MediaListStatus::Completed,
					"CURRENT" => MediaListStatus::Current,
					"PLANNING" => MediaListStatus::Planning,
					"DROPPED" => MediaListStatus::Dropped,
					"PAUSED" => MediaListStatus::Paused,
					_ => MediaListStatus::Repeating,
				};
				Some(UserStatusStatistic {
					count,
					status: Some(s),
				})
			})
			.collect()
	}

	#[test]
	fn get_color_blue() {
		let user = make_user(Some("blue"));
		assert_eq!(get_color(user), 0x3498DB);
	}

	#[test]
	fn get_color_purple() {
		let user = make_user(Some("purple"));
		assert_eq!(get_color(user), 0x9B59B6);
	}

	#[test]
	fn get_color_pink() {
		let user = make_user(Some("pink"));
		assert_eq!(get_color(user), 0xE68397);
	}

	#[test]
	fn get_color_orange() {
		let user = make_user(Some("orange"));
		assert_eq!(get_color(user), 0xE67E22);
	}

	#[test]
	fn get_color_red() {
		let user = make_user(Some("red"));
		assert_eq!(get_color(user), 0xE74C3C);
	}

	#[test]
	fn get_color_green() {
		let user = make_user(Some("green"));
		assert_eq!(get_color(user), 0x1F8B4C);
	}

	#[test]
	fn get_color_gray() {
		let user = make_user(Some("gray"));
		assert_eq!(get_color(user), 0x979C9F);
	}

	#[test]
	fn get_color_unknown_defaults_to_fabled_pink() {
		let user = make_user(Some("cyan"));
		assert_eq!(get_color(user), 0xFAB1ED);
	}

	#[test]
	fn get_color_none_defaults_to_fabled_pink() {
		let user = make_user(None);
		assert_eq!(get_color(user), 0xFAB1ED);
	}

	#[test]
	fn get_completed_finds_count() {
		let statuses = make_statuses(vec![
			("CURRENT", 5),
			("COMPLETED", 42),
			("DROPPED", 3),
		]);
		assert_eq!(get_completed(statuses), 42);
	}

	#[test]
	fn get_completed_zero_when_no_completed() {
		let statuses = make_statuses(vec![("CURRENT", 5), ("DROPPED", 3)]);
		assert_eq!(get_completed(statuses), 0);
	}

	#[test]
	fn get_completed_with_single_completed() {
		let statuses = make_statuses(vec![("COMPLETED", 100)]);
		assert_eq!(get_completed(statuses), 100);
	}

	#[test]
	fn get_tag_list_formats_correctly() {
		let tags = vec![
			Some(UserTagStatistic {
				tag: Some(MediaTag {
					name: "Action".to_string(),
				}),
			}),
			Some(UserTagStatistic {
				tag: Some(MediaTag {
					name: "Comedy".to_string(),
				}),
			}),
			Some(UserTagStatistic {
				tag: Some(MediaTag {
					name: "Drama".to_string(),
				}),
			}),
		];
		assert_eq!(get_tag_list(&tags),"Action/Comedy/Drama");
	}

	#[test]
	fn get_tag_list_caps_at_5() {
		let tags: Vec<Option<UserTagStatistic>> = (0..10)
			.map(|i| {
				Some(UserTagStatistic {
					tag: Some(MediaTag {
						name: format!("Tag{}", i),
					}),
				})
			})
			.collect();
		let result = get_tag_list(&tags);
		assert_eq!(result.split('/').count(), 5);
	}

	#[test]
	fn get_genre_list_formats_correctly() {
		let genres = vec![
			Some(UserGenreStatistic {
				genre: Some("Action".to_string()),
			}),
			Some(UserGenreStatistic {
				genre: Some("Romance".to_string()),
			}),
		];
		assert_eq!(get_genre_list(&genres),"Action/Romance");
	}

	#[test]
	fn get_genre_list_caps_at_5() {
		let genres: Vec<Option<UserGenreStatistic>> = (0..8)
			.map(|i| {
				Some(UserGenreStatistic {
					genre: Some(format!("Genre{}", i)),
				})
			})
			.collect();
		let result = get_genre_list(&genres);
		assert_eq!(result.split('/').count(), 5);
	}

	#[test]
	fn get_user_url_formats_correctly() {
		assert_eq!(get_user_url(&12345), "https://anilist.co/user/12345");
	}

	#[test]
	fn get_banner_formats_correctly() {
		assert_eq!(get_banner(&12345), "https://img.anili.st/user/12345");
	}

	#[test]
	fn get_user_manga_url_formats_correctly() {
		assert_eq!(
			get_user_manga_url(12345),
			"https://anilist.co/user/12345/mangalist"
		);
	}

	#[test]
	fn get_user_anime_url_formats_correctly() {
		assert_eq!(
			get_user_anime_url(12345),
			"https://anilist.co/user/12345/animelist"
		);
	}

	#[test]
	fn media_list_status_display() {
		assert_eq!(MediaListStatus::Current.to_string(), "CURRENT");
		assert_eq!(MediaListStatus::Planning.to_string(), "PLANNING");
		assert_eq!(MediaListStatus::Completed.to_string(), "COMPLETED");
		assert_eq!(MediaListStatus::Dropped.to_string(), "DROPPED");
		assert_eq!(MediaListStatus::Paused.to_string(), "PAUSED");
		assert_eq!(MediaListStatus::Repeating.to_string(), "REPEATING");
	}
}
