use std::fmt::{Display, Write};
use std::sync::Arc;

use anyhow::{Context, Result};
use cynic::{GraphQlResponse, QueryBuilder};

use crate::anilist::make_request::make_request_anilist;
use crate::cache::CacheInterface;
use crate::database::prelude::RegisteredUser;
use sea_orm::{DatabaseConnection, EntityTrait};

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MediaQuerryIdVariables {
	pub format_in: Option<Vec<Option<MediaFormat>>>,
	pub id: Option<i32>,
	pub media_type: Option<MediaType>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaQuerryIdVariables")]
pub struct MediaQuerryId {
	#[arguments(type: $ media_type, id: $ id, format_in: $ format_in)]
	#[cynic(rename = "Media")]
	pub media: Option<Media>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MediaQuerrySearchVariables<'a> {
	pub format_in: Option<Vec<Option<MediaFormat>>>,
	pub media_type: Option<MediaType>,
	pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaQuerrySearchVariables")]
pub struct MediaQuerrySearch {
	#[arguments(search: $ search, type: $ media_type, format_in: $ format_in)]
	#[cynic(rename = "Media")]
	pub media: Option<Media>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
	pub id: i32,
	pub cover_image: Option<MediaCoverImage>,
	pub title: Option<MediaTitle>,
	pub source: Option<MediaSource>,
	pub site_url: Option<String>,
	pub genres: Option<Vec<Option<String>>>,
	pub favourites: Option<i32>,
	pub format: Option<MediaFormat>,
	pub duration: Option<i32>,
	pub staff: Option<StaffConnection>,
	pub start_date: Option<FuzzyDate>,
	pub end_date: Option<FuzzyDate>,
	pub chapters: Option<i32>,
	pub characters: Option<CharacterConnection>,
	pub tags: Option<Vec<Option<MediaTag>>>,
	pub external_links: Option<Vec<Option<MediaExternalLink>>>,
	pub trailer: Option<MediaTrailer>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffConnection {
	pub edges: Option<Vec<Option<StaffEdge>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffEdge {
	pub role: Option<String>,
	pub node: Option<Staff>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Staff {
	pub name: Option<StaffName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct StaffName {
	pub user_preferred: Option<String>,
	pub full: Option<String>,
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
pub struct FuzzyDate {
	pub year: Option<i32>,
	pub month: Option<i32>,
	pub day: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterConnection {
	pub edges: Option<Vec<Option<CharacterEdge>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterEdge {
	pub node: Option<Character>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Character {
	pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CharacterName {
	pub user_preferred: Option<String>,
	pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTag {
	pub name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaExternalLink {
	pub url: Option<String>,
	pub site: String,
	#[cynic(rename = "type")]
	pub link_type: Option<ExternalLinkType>,
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
pub enum ExternalLinkType {
	Info,
	Streaming,
	Social,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTrailer {
	pub id: Option<String>,
	pub site: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
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
pub enum ScoreFormat {
	#[cynic(rename = "POINT_100")]
	Point100,
	#[cynic(rename = "POINT_10_DECIMAL")]
	Point10Decimal,
	#[cynic(rename = "POINT_10")]
	Point10,
	#[cynic(rename = "POINT_5")]
	Point5,
	#[cynic(rename = "POINT_3")]
	Point3,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MediaListScoreVariables {
	pub user_id_in: Option<Vec<Option<i32>>>,
	pub media_id: Option<i32>,
	pub per_page: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaListScoreVariables")]
pub struct MediaListScoreQuery {
	#[arguments(perPage: $per_page)]
	#[cynic(rename = "Page")]
	pub page: Option<MediaListScorePage>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Page", variables = "MediaListScoreVariables")]
pub struct MediaListScorePage {
	#[arguments(userId_in: $user_id_in, mediaId: $media_id)]
	pub media_list: Option<Vec<Option<MediaListEntry>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "MediaList")]
pub struct MediaListEntry {
	#[arguments(format: POINT_10_DECIMAL)]
	pub score: Option<f64>,
	pub status: Option<MediaListStatus>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CharacterRole {
	Main,
	Supporting,
	Background,
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
#[allow(dead_code)]
pub enum MediaSeason {
	Winter,
	Spring,
	Summer,
	Fall,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaSource {
	Original,
	Manga,
	LightNovel,
	VisualNovel,
	VideoGame,
	Other,
	Novel,
	Doujinshi,
	Anime,
	WebNovel,
	LiveAction,
	Game,
	Comic,
	MultimediaProject,
	PictureBook,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum MediaStatus {
	Finished,
	Releasing,
	NotYetReleased,
	Cancelled,
	Hiatus,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaType {
	Anime,
	Manga,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[allow(dead_code)]
pub struct CountryCode(pub String);

impl Display for CountryCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.clone())
	}
}

impl Display for MediaType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaType::Anime => write!(f, "Anime"),
			MediaType::Manga => write!(f, "Manga"),
		}
	}
}

impl Display for MediaStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaStatus::Finished => write!(f, "Finished"),
			MediaStatus::Releasing => write!(f, "Releasing"),
			MediaStatus::NotYetReleased => write!(f, "Not Yet Released"),
			MediaStatus::Cancelled => write!(f, "Cancelled"),
			MediaStatus::Hiatus => write!(f, "Hiatus"),
		}
	}
}

impl Display for MediaSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaSource::Original => write!(f, "Original"),
			MediaSource::Manga => write!(f, "Manga"),
			MediaSource::LightNovel => write!(f, "Light Novel"),
			MediaSource::VisualNovel => write!(f, "Visual Novel"),
			MediaSource::VideoGame => write!(f, "Video Game"),
			MediaSource::Other => write!(f, "Other"),
			MediaSource::Novel => write!(f, "Novel"),
			MediaSource::Doujinshi => write!(f, "Doujinshi"),
			MediaSource::Anime => write!(f, "Anime"),
			MediaSource::WebNovel => write!(f, "Web Novel"),
			MediaSource::LiveAction => write!(f, "Live Action"),
			MediaSource::Game => write!(f, "Game"),
			MediaSource::Comic => write!(f, "Comic"),
			MediaSource::MultimediaProject => write!(f, "Multimedia Project"),
			MediaSource::PictureBook => write!(f, "Picture Book"),
		}
	}
}

impl Display for MediaFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaFormat::Tv => write!(f, "TV"),
			MediaFormat::TvShort => write!(f, "TV Short"),
			MediaFormat::Movie => write!(f, "Movie"),
			MediaFormat::Special => write!(f, "Special"),
			MediaFormat::Ova => write!(f, "OVA"),
			MediaFormat::Ona => write!(f, "ONA"),
			MediaFormat::Music => write!(f, "Music"),
			MediaFormat::Manga => write!(f, "Manga"),
			MediaFormat::Novel => write!(f, "Novel"),
			MediaFormat::OneShot => write!(f, "One Shot"),
		}
	}
}

impl Display for MediaSeason {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MediaSeason::Winter => write!(f, "Winter"),
			MediaSeason::Spring => write!(f, "Spring"),
			MediaSeason::Summer => write!(f, "Summer"),
			MediaSeason::Fall => write!(f, "Fall"),
		}
	}
}

impl Display for CharacterRole {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CharacterRole::Main => write!(f, "Main"),
			CharacterRole::Supporting => write!(f, "Supporting"),
			CharacterRole::Background => write!(f, "Background"),
		}
	}
}

pub fn embed_title(title: &MediaTitle) -> String {
	let en = title.english.clone().unwrap_or_default();
	let rj = title.romaji.clone().unwrap_or_default();

	let mut title = String::new();
	let mut has_en_title = false;

	match en.as_str() {
		"" => {},
		_ => {
			has_en_title = true;
			title.push_str(en.as_str())
		},
	}

	match rj.as_str() {
		"" => {},
		_ => {
			if has_en_title {
				title.push_str(" / ");
				title.push_str(rj.as_str())
			} else {
				title.push_str(rj.as_str())
			}
		},
	}

	title
}

pub fn get_media_url(media: &Media) -> String {
	media
		.site_url
		.clone()
		.unwrap_or("https://example.com".to_string())
}

pub fn get_banner(media: &Media) -> String {
	format!("https://img.anili.st/media/{}", media.id)
}

pub fn get_staff(staff: Vec<Option<StaffEdge>>) -> String {
	let mut staff_text = String::new();
	let mut i = 0;

	for s in staff.into_iter() {
		if i > 4 {
			break;
		}

		let s = match s {
			Some(s) => s,
			None => continue,
		};

		let node = match s.node.clone() {
			Some(n) => n,
			None => continue,
		};

		let name = match node.name {
			Some(n) => n,
			None => continue,
		};

		let full = name.full;
		let user_pref = name.user_preferred;
		let staff_name = user_pref.unwrap_or(full.unwrap_or("Unknown".to_string()));

		let s_role = s.role.clone();
		let role = s_role.unwrap_or("Unknown".to_string());

		write!(staff_text, "{}: {}\n", staff_name, role).unwrap();
		i += 1;
	}

	staff_text
}

pub fn get_characters(characters: Vec<Option<CharacterEdge>>) -> String {
	let mut characters_text = String::new();
	let mut i = 0;

	for c in characters.into_iter() {
		if i > 4 {
			break;
		}

		let c = match c {
			Some(c) => c,
			None => continue,
		};

		let node = match c.node.clone() {
			Some(n) => n,
			None => continue,
		};

		let name = match node.name {
			Some(n) => n,
			None => continue,
		};

		let full = name.full;
		let user_pref = name.user_preferred;
		let char_name = user_pref.unwrap_or(full.unwrap_or("Unknown".to_string()));

		writeln!(characters_text, "{}", char_name).unwrap();
		i += 1;
	}

	characters_text
}

/// Extract streaming platform links from external links.
pub fn get_streaming_links(external_links: &Option<Vec<Option<MediaExternalLink>>>) -> String {
	let links = match external_links {
		Some(links) => links,
		None => return String::new(),
	};

	let streaming: Vec<String> = links
		.iter()
		.flatten()
		.filter(|link| link.link_type == Some(ExternalLinkType::Streaming) && link.url.is_some())
		.map(|link| format!("[{}]({})", link.site, link.url.as_ref().unwrap()))
		.collect();

	streaming.join(" | ")
}

/// Build a trailer URL from a MediaTrailer.
pub fn get_trailer_url(trailer: &Option<MediaTrailer>) -> Option<String> {
	let trailer = trailer.as_ref()?;
	let id = trailer.id.as_ref().filter(|id| !id.is_empty())?;
	let site = trailer.site.as_ref().filter(|s| !s.is_empty())?;

	match site.as_str() {
		"youtube" => Some(format!("https://www.youtube.com/watch?v={}", id)),
		"dailymotion" => Some(format!("https://www.dailymotion.com/video/{}", id)),
		_ => None,
	}
}

/// Format guild member scores into a summary string.
pub fn format_guild_scores(scores: &[MediaListEntry]) -> Option<String> {
	let with_score: Vec<f64> = scores
		.iter()
		.filter_map(|e| e.score)
		.filter(|&s| s > 0.0)
		.collect();

	if with_score.is_empty() {
		return None;
	}

	let count = with_score.len();
	let avg = with_score.iter().sum::<f64>() / count as f64;

	Some(format!("{} scored | Avg: {:.1}/10", count, avg))
}

/// Fetch guild media scores from AniList for a list of user IDs.
pub async fn get_guild_media_scores(
	media_id: i32, anilist_ids: Vec<i32>, anilist_cache: Arc<CacheInterface>,
) -> Result<Vec<MediaListEntry>> {
	if anilist_ids.is_empty() {
		return Ok(Vec::new());
	}

	let var = MediaListScoreVariables {
		user_id_in: Some(anilist_ids.into_iter().map(Some).collect()),
		media_id: Some(media_id),
		per_page: Some(50),
	};

	let operation = MediaListScoreQuery::build(var);
	let data: GraphQlResponse<MediaListScoreQuery> =
		make_request_anilist(operation, false, anilist_cache).await?;

	Ok(data
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.media_list)
		.map(|list| list.into_iter().flatten().collect())
		.unwrap_or_default())
}

/// Get all registered AniList user IDs from the database.
pub async fn get_registered_anilist_ids(db: &DatabaseConnection) -> Result<Vec<i32>> {
	let users = RegisteredUser::find().all(db).await?;
	Ok(users.into_iter().map(|u| u.anilist_id).collect())
}

/// Fetch AniList media by ID or name.
pub async fn get_media(
	value: &str, media_type: Option<MediaType>, format_in: Option<Vec<Option<MediaFormat>>>,
	anilist_cache: Arc<CacheInterface>,
) -> Result<Media> {
	if let Ok(id) = value.parse::<i32>() {
		let var = MediaQuerryIdVariables {
			format_in: format_in.clone(),
			id: Some(id),
			media_type,
		};
		let operation = MediaQuerryId::build(var);
		let data: GraphQlResponse<MediaQuerryId> =
			make_request_anilist(operation, true, anilist_cache).await?;
		data.data
			.context("No data from AniList")?
			.media
			.context(format!("No media found with ID {}", id))
	} else {
		let var = MediaQuerrySearchVariables {
			format_in,
			media_type,
			search: Some(value),
		};
		let operation = MediaQuerrySearch::build(var);
		let data: GraphQlResponse<MediaQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;
		data.data
			.context("No data from AniList")?
			.media
			.context(format!("No media found with name '{}'", value))
	}
}
