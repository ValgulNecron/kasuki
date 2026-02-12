#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MinimalAnimeIdVariables {
	pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MinimalAnimeIdVariables")]
pub struct MinimalAnimeId {
	#[arguments(id: $ id, type: "ANIME")]
	#[cynic(rename = "Media")]
	pub media: Option<Media>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MinimalAnimeSearchVariables<'a> {
	pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MinimalAnimeSearchVariables")]
pub struct MinimalAnimeSearch {
	#[arguments(search: $ search, type: "ANIME")]
	#[cynic(rename = "Media")]
	pub media: Option<Media>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
	pub id: i32,
	pub cover_image: Option<MediaCoverImage>,
	pub title: Option<MediaTitle>,
	pub next_airing_episode: Option<AiringSchedule>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
	pub english: Option<String>,
	pub romaji: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaCoverImage {
	pub extra_large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct AiringSchedule {
	pub airing_at: i32,
	pub episode: i32,
}

use crate::anilist::make_request::make_request_anilist;
use crate::cache::CacheInterface;
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::trace;

pub async fn get_minimal_anime_by_id(id: i32, cache: Arc<RwLock<CacheInterface>>) -> Result<Media> {
	trace!(?id);

	let query = MinimalAnimeIdVariables { id: Some(id) };

	let operation = MinimalAnimeId::build(query);

	let response: GraphQlResponse<MinimalAnimeId> =
		make_request_anilist(operation, true, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

pub async fn get_minimal_anime_by_search(
	query: &str, cache: Arc<RwLock<CacheInterface>>,
) -> Result<Media> {
	trace!(?query);

	let search_query = MinimalAnimeSearchVariables {
		search: Some(query),
	};

	let operation = MinimalAnimeSearch::build(search_query);

	let response: GraphQlResponse<MinimalAnimeSearch> =
		make_request_anilist(operation, true, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

pub async fn get_minimal_anime_media(
	anime: String, cache: Arc<RwLock<CacheInterface>>,
) -> Result<Media> {
	let media = if let Ok(id) = anime.parse::<i32>() {
		get_minimal_anime_by_id(id, cache).await?
	} else {
		get_minimal_anime_by_search(&anime, cache).await?
	};

	trace!(?media);

	Ok(media)
}
