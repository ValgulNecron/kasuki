use std::fmt::Display;
use std::sync::Arc;

use anyhow::Result;
use cynic::{GraphQlResponse, QueryBuilder};

use crate::anilist::make_request::make_request_anilist;
use crate::cache::CacheInterface;

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
#[allow(dead_code)]
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

/// Fetch an AniList user by ID or username.
pub async fn get_user(value: &str, anilist_cache: Arc<CacheInterface>) -> Result<User> {
	let user = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;
		let var = UserQueryIdVariables { id: Some(id) };
		let operation = UserQueryId::build(var);
		let data: GraphQlResponse<UserQueryId> =
			make_request_anilist(operation, true, anilist_cache).await?;
		data.data.unwrap().user.unwrap()
	} else {
		let var = UserQuerySearchVariables {
			search: Some(value),
		};
		let operation = UserQuerySearch::build(var);
		let data: GraphQlResponse<UserQuerySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;
		data.data.unwrap().user.unwrap()
	};
	Ok(user)
}

pub fn get_user_url(user_id: &i32) -> String {
	format!("https://anilist.co/user/{}", user_id)
}

pub fn get_banner(user_id: &i32) -> String {
	format!("https://img.anili.st/user/{}", user_id)
}

pub fn get_color(user: User) -> u32 {
	match user
		.options
		.unwrap()
		.profile_color
		.clone()
		.unwrap_or_else(|| "#FF00FF".to_string())
		.as_str()
	{
		"blue" => 0x3498DB,
		"purple" => 0x9B59B6,
		"pink" => 0xE68397,
		"orange" => 0xE67E22,
		"red" => 0xE74C3C,
		"green" => 0x1F8B4C,
		"gray" => 0x979C9F,
		_ => 0xFAB1ED,
	}
}

pub fn get_completed(statuses: Vec<Option<UserStatusStatistic>>) -> i32 {
	let mut completed = 0;
	for i in statuses {
		let i = i.unwrap();
		if i.status.unwrap().to_string() == *"COMPLETED" {
			completed = i.count;
		}
	}
	completed
}

pub fn get_tag_list(vec: &[Option<UserTagStatistic>]) -> String {
	vec.iter()
		.filter_map(|tag| Some(tag.as_ref()?.tag.as_ref()?.name.as_str()))
		.take(5)
		.collect::<Vec<_>>()
		.join("/")
}

pub fn get_genre_list(vec: &[Option<UserGenreStatistic>]) -> String {
	vec.iter()
		.filter_map(|genre| genre.as_ref()?.genre.as_deref())
		.take(5)
		.collect::<Vec<_>>()
		.join("/")
}
