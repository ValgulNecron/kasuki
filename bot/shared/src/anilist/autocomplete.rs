use std::sync::Arc;

use anyhow::Result;
use cynic::QueryBuilder;

use crate::anilist::make_request::make_request_anilist;
use crate::cache::CacheInterface;

pub mod character {
	#[cynic::schema("anilist")]
	mod schema {}

	#[derive(cynic::QueryVariables, Debug, Clone)]
	pub struct CharacterAutocompleteVariables<'a> {
		pub search: Option<&'a str>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", variables = "CharacterAutocompleteVariables")]
	pub struct CharacterAutocomplete {
		#[arguments(perPage: 25)]
		#[cynic(rename = "Page")]
		pub page: Option<Page>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(variables = "CharacterAutocompleteVariables")]
	pub struct Page {
		#[arguments(search: $ search)]
		pub characters: Option<Vec<Option<Character>>>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Character {
		pub id: i32,
		pub name: Option<CharacterName>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct CharacterName {
		pub full: Option<String>,
		pub user_preferred: Option<String>,
		pub native: Option<String>,
	}
}

pub mod user {
	#[cynic::schema("anilist")]
	mod schema {}

	#[derive(cynic::QueryVariables, Debug, Clone)]
	pub struct UserAutocompleteVariables<'a> {
		pub search: Option<&'a str>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", variables = "UserAutocompleteVariables")]
	pub struct UserAutocomplete {
		#[arguments(perPage: 25)]
		#[cynic(rename = "Page")]
		pub page: Option<Page>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(variables = "UserAutocompleteVariables")]
	pub struct Page {
		#[arguments(search: $ search)]
		pub users: Option<Vec<Option<User>>>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct User {
		pub id: i32,
		pub name: String,
	}
}

pub mod staff {
	#[cynic::schema("anilist")]
	mod schema {}

	#[derive(cynic::QueryVariables, Debug, Clone)]
	pub struct StaffAutocompleteVariables<'a> {
		pub search: Option<&'a str>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", variables = "StaffAutocompleteVariables")]
	pub struct StaffAutocomplete {
		#[arguments(perPage: 25)]
		#[cynic(rename = "Page")]
		pub page: Option<Page>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(variables = "StaffAutocompleteVariables")]
	pub struct Page {
		#[arguments(search: $ search)]
		pub staff: Option<Vec<Option<Staff>>>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Staff {
		pub id: i32,
		pub name: Option<StaffName>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct StaffName {
		pub native: Option<String>,
		pub user_preferred: Option<String>,
		pub full: Option<String>,
	}
}

pub mod studio {
	#[cynic::schema("anilist")]
	mod schema {}

	#[derive(cynic::QueryVariables, Debug, Clone)]
	pub struct StudioAutocompleteVariables<'a> {
		pub search: Option<&'a str>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", variables = "StudioAutocompleteVariables")]
	pub struct StudioAutocomplete {
		#[arguments(perPage: 25)]
		#[cynic(rename = "Page")]
		pub page: Option<Page>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(variables = "StudioAutocompleteVariables")]
	pub struct Page {
		#[arguments(search: $ search)]
		pub studios: Option<Vec<Option<Studio>>>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Studio {
		pub name: String,
		pub id: i32,
	}
}

pub mod media {
	#[cynic::schema("anilist")]
	mod schema {}

	#[derive(cynic::QueryVariables, Debug, Clone)]
	pub struct MediaAutocompleteVariables<'a> {
		pub in_media_format: Option<Vec<Option<MediaFormat>>>,
		pub media_type: Option<MediaType>,
		pub search: Option<&'a str>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", variables = "MediaAutocompleteVariables")]
	pub struct MediaAutocomplete {
		#[arguments(perPage: 25)]
		#[cynic(rename = "Page")]
		pub page: Option<Page>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(variables = "MediaAutocompleteVariables")]
	pub struct Page {
		#[arguments(search: $ search, type: $ media_type, format_in: $ in_media_format)]
		pub media: Option<Vec<Option<Media>>>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Media {
		pub id: i32,
		pub title: Option<MediaTitle>,
	}

	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct MediaTitle {
		pub user_preferred: Option<String>,
		pub romaji: Option<String>,
		pub native: Option<String>,
	}

	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaFormat {
		Tv,
		TvShort,
		Movie,
		Special,
		Ova,
		Ona,
		Music,
		Manga,
		Novel,
		OneShot,
	}

	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaType {
		Anime,
		Manga,
	}
}

/// Search AniList characters by name, returns (display_name, id) pairs.
pub async fn search_characters(
	search: &str, cache: Arc<CacheInterface>,
) -> Result<Vec<(String, String)>> {
	use character::*;
	let var = CharacterAutocompleteVariables {
		search: Some(search),
	};
	let operation = CharacterAutocomplete::build(var);
	let data: cynic::GraphQlResponse<CharacterAutocomplete> =
		make_request_anilist(operation, true, cache).await?;

	let characters = data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.characters)
		.unwrap_or_default();

	Ok(characters
		.into_iter()
		.flatten()
		.map(|c| {
			let name = c
				.name
				.and_then(|n| n.user_preferred.or(n.full).or(n.native))
				.unwrap_or_default();
			(name, c.id.to_string())
		})
		.collect())
}

/// Search AniList users by name, returns (display_name, id) pairs.
pub async fn search_users(
	search: &str, cache: Arc<CacheInterface>,
) -> Result<Vec<(String, String)>> {
	use user::*;
	let var = UserAutocompleteVariables {
		search: Some(search),
	};
	let operation = UserAutocomplete::build(var);
	let data: cynic::GraphQlResponse<UserAutocomplete> =
		make_request_anilist(operation, true, cache).await?;

	let users = data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.users)
		.unwrap_or_default();

	Ok(users
		.into_iter()
		.flatten()
		.map(|u| (u.name, u.id.to_string()))
		.collect())
}

/// Search AniList staff by name, returns (display_name, id) pairs.
pub async fn search_staff(
	search: &str, cache: Arc<CacheInterface>,
) -> Result<Vec<(String, String)>> {
	use staff::*;
	let var = StaffAutocompleteVariables {
		search: Some(search),
	};
	let operation = StaffAutocomplete::build(var);
	let data: cynic::GraphQlResponse<StaffAutocomplete> =
		make_request_anilist(operation, true, cache).await?;

	let results = data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.staff)
		.unwrap_or_default();

	Ok(results
		.into_iter()
		.flatten()
		.map(|s| {
			let name = s
				.name
				.and_then(|n| n.user_preferred.or(n.native).or(n.full))
				.unwrap_or_default();
			(name, s.id.to_string())
		})
		.collect())
}

/// Search AniList studios by name, returns (display_name, id) pairs.
pub async fn search_studios(
	search: &str, cache: Arc<CacheInterface>,
) -> Result<Vec<(String, String)>> {
	use studio::*;
	let var = StudioAutocompleteVariables {
		search: Some(search),
	};
	let operation = StudioAutocomplete::build(var);
	let data: cynic::GraphQlResponse<StudioAutocomplete> =
		make_request_anilist(operation, true, cache).await?;

	let studios = data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.studios)
		.unwrap_or_default();

	Ok(studios
		.into_iter()
		.flatten()
		.map(|s| (s.name, s.id.to_string()))
		.collect())
}

/// Search AniList media by title with format/type filters, returns (display_title, id) pairs.
pub async fn search_media(
	search: &str, media_type: Option<media::MediaType>,
	formats: Option<Vec<Option<media::MediaFormat>>>, cache: Arc<CacheInterface>,
) -> Result<Vec<(String, String)>> {
	use media::*;
	let var = MediaAutocompleteVariables {
		search: Some(search),
		media_type,
		in_media_format: formats,
	};
	let operation = MediaAutocomplete::build(var);
	let data: cynic::GraphQlResponse<MediaAutocomplete> =
		make_request_anilist(operation, true, cache).await?;

	let medias = data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.media)
		.unwrap_or_default();

	Ok(medias
		.into_iter()
		.flatten()
		.filter_map(|m| {
			let title = m
				.title
				.and_then(|t| t.user_preferred.or(t.romaji).or(t.native))?;
			Some((title, m.id.to_string()))
		})
		.collect())
}
