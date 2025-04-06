use std::fmt::Display;

use crate::command::command_trait::{EmbedContent, EmbedType};
use crate::config::DbConfig;
use crate::constant::COLOR;
use crate::structure::message::anilist_user::user::{UserLocalised, load_localization_user};
use anyhow::{Result, anyhow};
use serenity::all::CommandInteraction;
use serenity::model::Colour;
use serenity::prelude::Context as SerenityContext;

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
	ctx: &'a SerenityContext, command: &'a CommandInteraction, user: User, db_config: DbConfig,
) -> Result<EmbedContent<'a, 'a>> {
	let guild_id = match command.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let user_localised = load_localization_user(guild_id, db_config).await?;

	let mut field = Vec::new();

	let statistics = user
		.statistics
		.clone()
		.ok_or(anyhow!("Could not get the statistics"))?;

	let manga = statistics.manga.clone();

	let anime = statistics.anime.clone();

	if let Some(m) = &manga {
		if m.count > 0 {
			field.push(get_manga_field(user.id, user_localised.clone(), m.clone()))
		}
	}

	if let Some(a) = &anime {
		if a.count > 0 {
			field.push(get_anime_field(user.id, user_localised.clone(), a.clone()))
		}
	}

	let mut content = EmbedContent {
		title: user.name.clone(),
		description: "".to_string(),
		thumbnail: None,
		url: Some(get_user_url(&user.id)),
		command_type: EmbedType::First,
		colour: Some(get_color(user.clone())),
		fields: field,
		images: None,
		action_row: None,
		images_url: Some(get_banner(&user.id)),
	};

	if let Some(avatar) = user.avatar {
		if let Some(large) = avatar.large {
			content.thumbnail = Some(large)
		}
	}

	Ok(content)
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
	user_id: i32, localised: UserLocalised, manga: UserStatistics2,
) -> (String, String, bool) {
	(
		String::new(),
		get_manga_desc(manga, localised, user_id),
		false,
	)
}

fn get_anime_field(
	user_id: i32, localised: UserLocalised, anime: UserStatistics,
) -> (String, String, bool) {
	(
		String::new(),
		get_anime_desc(anime, localised, user_id),
		false,
	)
}

fn get_manga_desc(manga: UserStatistics2, localised: UserLocalised, user_id: i32) -> String {
	localised
		.manga
		.replace("$url$", get_user_manga_url(user_id).as_str())
		.replace("$count$", manga.count.to_string().as_str())
		.replace(
			"$complete$",
			get_completed(manga.statuses.unwrap().clone())
				.to_string()
				.as_str(),
		)
		.replace("$chap$", manga.chapters_read.to_string().as_str())
		.replace("$score$", manga.mean_score.to_string().as_str())
		.replace("$sd$", manga.standard_deviation.to_string().as_str())
		.replace(
			"$tag_list$",
			get_tag_list(manga.tags.clone().unwrap()).as_str(),
		)
		.replace(
			"$genre_list$",
			get_genre_list(manga.genres.clone().unwrap()).as_str(),
		)
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

fn get_anime_desc(anime: UserStatistics, localised: UserLocalised, user_id: i32) -> String {
	localised
		.anime
		.replace("$url$", get_user_anime_url(user_id).as_str())
		.replace("$count$", anime.count.to_string().as_str())
		.replace(
			"$complete$",
			get_completed(anime.statuses.clone().unwrap())
				.to_string()
				.as_str(),
		)
		.replace(
			"$duration$",
			get_anime_time_watch(anime.minutes_watched, localised.clone()).as_str(),
		)
		.replace("$score$", anime.mean_score.to_string().as_str())
		.replace("$sd$", anime.standard_deviation.to_string().as_str())
		.replace(
			"$tag_list$",
			get_tag_list(anime.tags.clone().unwrap()).as_str(),
		)
		.replace(
			"$genre_list$",
			get_genre_list(anime.genres.clone().unwrap()).as_str(),
		)
}

fn get_anime_time_watch(i: i32, localised1: UserLocalised) -> String {
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

	let weeks = match week {
		1 => format!("{} {}", localised1.week, week),
		_ => format!("{} {}", localised1.weeks, week),
	};

	tw.push_str(weeks.as_str());

	let days = match days {
		1 => format!("{} {}", localised1.day, days),
		_ => format!("{} {}", localised1.days, days),
	};

	tw.push_str(days.as_str());

	let hours = match hour {
		1 => format!("{} {}", localised1.hour, hour),
		_ => format!("{} {}", localised1.hours, hour),
	};

	tw.push_str(hours.as_str());

	let mins = match min {
		1 => format!("{} {}", localised1.minute, min),
		_ => format!("{} {}", localised1.minutes, min),
	};

	tw.push_str(mins.as_str());

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
