use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::constant::COLOR;
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use sea_orm::DatabaseConnection;
use serenity::all::CommandInteraction;
use serenity::model::Colour;
use shared::localization::{get_language_identifier, LanguageIdentifier, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

#[cynic::schema("anilist")]

mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct UserQueryIdVariables {
	pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "UserQueryIdVariables")]

pub struct UserQueryId {
	#[arguments(id: $ id)]
	#[cynic(rename = "User")]
	pub user: Option<User>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]

pub struct UserQuerySearchVariables<'a> {
	pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "UserQuerySearchVariables")]

pub struct UserQuerySearch {
	#[arguments(search: $ search)]
	#[cynic(rename = "User")]
	pub user: Option<User>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct User {
	pub id: i32,
	pub name: String,
	pub avatar: Option<UserAvatar>,
	pub statistics: Option<UserStatisticTypes>,
	pub options: Option<UserOptions>,
	pub banner_image: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserOptions {
	pub profile_color: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserStatisticTypes {
	pub anime: Option<UserStatistics>,
	pub manga: Option<UserStatistics2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "UserStatistics")]

pub struct UserStatistics2 {
	pub count: i32,
	pub mean_score: f64,
	pub standard_deviation: f64,
	pub chapters_read: i32,
	#[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
	pub tags: Option<Vec<Option<UserTagStatistic>>>,
	#[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
	pub genres: Option<Vec<Option<UserGenreStatistic>>>,
	#[arguments(sort: "COUNT_DESC")]
	pub statuses: Option<Vec<Option<UserStatusStatistic>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserStatistics {
	pub count: i32,
	pub mean_score: f64,
	pub standard_deviation: f64,
	pub minutes_watched: i32,
	#[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
	pub tags: Option<Vec<Option<UserTagStatistic>>>,
	#[arguments(limit: 5, sort: "MEAN_SCORE_DESC")]
	pub genres: Option<Vec<Option<UserGenreStatistic>>>,
	#[arguments(sort: "COUNT_DESC")]
	pub statuses: Option<Vec<Option<UserStatusStatistic>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserStatusStatistic {
	pub count: i32,
	pub status: Option<MediaListStatus>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserGenreStatistic {
	pub genre: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserTagStatistic {
	pub tag: Option<MediaTag>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct UserAvatar {
	pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaTag {
	pub name: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum MediaListStatus {
	Current,
	Planning,
	Completed,
	Dropped,
	Paused,
	Repeating,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

pub enum UserStatisticsSort {
	Id,
	IdDesc,
	Count,
	CountDesc,
	Progress,
	ProgressDesc,
	MeanScore,
	MeanScoreDesc,
}

impl Display for MediaListStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaListStatus::Current => write!(f, "CURRENT"),
			MediaListStatus::Planning => write!(f, "PLANNING"),
			MediaListStatus::Completed => write!(f, "COMPLETED"),
			MediaListStatus::Dropped => write!(f, "DROPPED"),
			MediaListStatus::Paused => write!(f, "PAUSED"),
			MediaListStatus::Repeating => write!(f, "REPEATING"),
		}
	}
}

impl Display for UserStatisticsSort {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			UserStatisticsSort::Id => write!(f, "ID"),
			UserStatisticsSort::IdDesc => write!(f, "ID_DESC"),
			UserStatisticsSort::Count => write!(f, "COUNT"),
			UserStatisticsSort::CountDesc => write!(f, "COUNT_DESC"),
			UserStatisticsSort::Progress => write!(f, "PROGRESS"),
			UserStatisticsSort::ProgressDesc => write!(f, "PROGRESS_DESC"),
			UserStatisticsSort::MeanScore => write!(f, "MEAN_SCORE"),
			UserStatisticsSort::MeanScoreDesc => write!(f, "MEAN_SCORE_DESC"),
		}
	}
}

pub async fn user_content<'a>(
	command: CommandInteraction, user: User, db_connection: Arc<DatabaseConnection>,
) -> Result<EmbedsContents<'a>> {
	let guild_id = match command.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let mut field = Vec::new();

	let statistics = user
		.statistics
		.clone()
		.ok_or(anyhow!("Could not get the statistics"))?;

	let manga = statistics.manga.clone();

	let anime = statistics.anime.clone();

	if let Some(m) = &manga {
		if m.count > 0 {
			field.push(get_manga_field(user.id, &lang_id, m.clone()))
		}
	}

	if let Some(a) = &anime {
		if a.count > 0 {
			field.push(get_anime_field(user.id, &lang_id, a.clone()))
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

	let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

	Ok(embed_contents)
}

pub fn get_user_url(user_id: &i32) -> String {
	format!("https://anilist.co/user/{}", user_id)
}

pub fn get_banner(user_id: &i32) -> String {
	format!("https://img.anili.st/user/{}", user_id)
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
	desc.push_str(USABLE_LOCALES.lookup_with_args(lang_id, "anilist_user_user-manga-title", &args).as_str());
	desc = desc.replace("\u{2069}", "");
	desc = desc.replace("\u{2068}", "");
	desc.push_str("\n");

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
		FluentValue::from(get_tag_list(manga.tags.clone().unwrap())),
	);
	args.insert(
		Cow::Borrowed("genre_list"),
		FluentValue::from(get_genre_list(manga.genres.clone().unwrap())),
	);


	desc.push_str(USABLE_LOCALES.lookup_with_args(lang_id, "anilist_user_user-manga", &args).as_str());
	desc
}

fn get_tag_list(vec: Vec<Option<UserTagStatistic>>) -> String {
	let vec = vec
		.iter()
		.map(|tag| tag.clone().unwrap().tag.clone().unwrap().name.clone())
		.collect::<Vec<_>>();

	let vec = vec.into_iter().take(5).collect::<Vec<_>>();

	vec.join("/")
}

fn get_genre_list(vec: Vec<Option<UserGenreStatistic>>) -> String {
	let vec = vec
		.iter()
		.map(|genre| genre.clone().unwrap().genre.as_ref().unwrap().clone())
		.collect::<Vec<_>>();

	let vec = vec.into_iter().take(5).collect::<Vec<_>>();

	vec.join("/")
}

pub fn get_completed(statuses: Vec<Option<UserStatusStatistic>>) -> i32 {
	let anime_statuses = statuses;

	let mut anime_completed = 0;

	for i in anime_statuses {
		let i = i.unwrap();

		if i.status.unwrap().to_string() == *"COMPLETED" {
			anime_completed = i.count;
		}
	}

	anime_completed
}

fn get_anime_desc(anime: UserStatistics, lang_id: &LanguageIdentifier, user_id: i32) -> String {
	let mut desc = String::new();
	let mut args: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();
	args.insert(
		Cow::Borrowed("url"),
		FluentValue::from(get_user_anime_url(user_id)),
	);

	desc.push_str(USABLE_LOCALES.lookup_with_args(lang_id, "anilist_user_user-anime-title", &args).as_str());
	desc = desc.replace("\u{2069}", "");
	desc = desc.replace("\u{2068}", "");
	desc.push_str("\n");

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
		FluentValue::from(get_tag_list(anime.tags.clone().unwrap())),
	);
	args.insert(
		Cow::Borrowed("genre_list"),
		FluentValue::from(get_genre_list(anime.genres.clone().unwrap())),
	);
	desc.push_str(USABLE_LOCALES.lookup_with_args(lang_id, "anilist_user_user-anime", &args).as_str());
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

	if days >= 1
	{
		let day_label = match days {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-day"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-days"),
		};
		tw.push_str(&format!("{}{}", day_label, days));
	}

	if hour >= 1
	{
		let hour_label = match hour {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-hour"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-hours"),
		};
		tw.push_str(&format!("{}{}", hour_label, hour));
	}

	if min >= 1
	{
		let min_label = match min {
			1 => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-minute"),
			_ => USABLE_LOCALES.lookup(lang_id, "anilist_user_user-minutes"),
		};
		tw.push_str(&format!("{}{}", min_label, min));
	}
	tw
}

pub fn get_color(user: User) -> Colour {
	match user
		.options
		.unwrap()
		.profile_color
		.clone()
		.unwrap_or_else(|| "#FF00FF".to_string())
		.as_str()
	{
		"blue" => Colour::BLUE,
		"purple" => Colour::PURPLE,
		"pink" => Colour::MEIBE_PINK,
		"orange" => Colour::ORANGE,
		"red" => Colour::RED,
		"green" => Colour::DARK_GREEN,
		"gray" => Colour::LIGHT_GREY,
		_ => COLOR,
	}
}
