use std::fmt::Write;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use cynic::{GraphQlResponse, QueryBuilder};

use crate::anilist::make_request::make_request_anilist;
use crate::cache::CacheInterface;

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct CharacterQuerryIdVariables {
	pub id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "CharacterQuerryIdVariables")]
pub struct CharacterQuerryId {
	#[arguments(id: $ id)]
	#[cynic(rename = "Character")]
	pub character: Option<Character>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct CharacterQuerrySearchVariables<'a> {
	pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "CharacterQuerrySearchVariables")]
pub struct CharacterQuerrySearch {
	#[arguments(search: $ search)]
	#[cynic(rename = "Character")]
	pub character: Option<Character>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
	pub age: Option<String>,
	pub blood_type: Option<String>,
	pub date_of_birth: Option<FuzzyDate>,
	pub description: Option<String>,
	pub favourites: Option<i32>,
	pub gender: Option<String>,
	pub image: Option<CharacterImage>,
	pub name: Option<CharacterName>,
	pub site_url: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
	pub user_preferred: Option<String>,
	pub native: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterImage {
	pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct FuzzyDate {
	pub month: Option<i32>,
	pub year: Option<i32>,
	pub day: Option<i32>,
}

// Builds a date string in MM/DD/YYYY format, omitting missing parts.
// AniList FuzzyDates can have any combination of fields (e.g., month+day but no year for birthdays).
pub fn format_fuzzy_date(date: &FuzzyDate) -> String {
	let mut has_month: bool = false;
	let mut has_day: bool = false;
	let mut result = String::new();

	if let Some(m) = date.month {
		write!(result, "{:02}", m).unwrap();
		has_month = true
	}

	if let Some(d) = date.day {
		// Only add separator if there's a preceding component
		if has_month {
			result.push('/')
		}
		write!(result, "{:02}", d).unwrap();
		has_day = true
	}

	if let Some(y) = date.year {
		if has_day {
			result.push('/')
		}
		write!(result, "{:04}", y).unwrap();
	}

	result
}

// Shows both romanized and native (e.g., Japanese) names so users can identify the character in either script
pub fn format_character_name(name: &CharacterName) -> String {
	let native = name.native.clone().unwrap_or_default();
	let user_pref = name.user_preferred.clone().unwrap_or_default();
	format!("{}/{}", user_pref, native)
}

/// Fetch an AniList character by ID or name.
// Numeric strings are treated as IDs for exact lookup; everything else triggers a search query
pub async fn get_character(value: &str, anilist_cache: Arc<CacheInterface>) -> Result<Character> {
	if let Ok(id) = value.parse::<i32>() {
		get_character_by_id(id, anilist_cache).await
	} else {
		let var = CharacterQuerrySearchVariables {
			search: Some(value),
		};
		let operation = CharacterQuerrySearch::build(var);
		let data: GraphQlResponse<CharacterQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache)
				.await
				.context(format!(
					"Failed to make AniList API request for character search with query '{}'",
					value
				))?;
		data.data
			.context("No data returned from AniList API for character search")?
			.character
			.context(format!("No character found with name '{}'", value))
	}
}

/// Fetch an AniList character by ID.
pub async fn get_character_by_id(
	value: i32, anilist_cache: Arc<CacheInterface>,
) -> Result<Character> {
	let var = CharacterQuerryIdVariables { id: Some(value) };
	let operation = CharacterQuerryId::build(var);

	let data: GraphQlResponse<CharacterQuerryId> =
		make_request_anilist(operation, true, anilist_cache)
			.await
			.context(format!(
				"Failed to make AniList API request for character with ID {}",
				value
			))?;

	match data.data {
		Some(data) => match data.character {
			Some(media) => Ok(media),
			None => Err(anyhow!("No character found with ID {}", value)
				.context("The character ID may not exist or may have been removed from AniList")),
		},
		None => Err(anyhow!(
			"No data returned from AniList API for character with ID {}",
			value
		)
		.context("This could indicate an issue with the AniList API or the request format")),
	}
}
