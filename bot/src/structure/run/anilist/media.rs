use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::config::DbConfig;
use crate::constant::UNKNOWN;
use crate::database::anime_song::Column::AnilistId;
use crate::database::prelude::AnimeSong;
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::general_channel_info::get_nsfw;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::media::load_localization_media;
use anyhow::{Result, anyhow};
use sea_orm::{entity::*, query::*};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::fmt::Display;
use std::sync::Arc;

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
	pub volumes: Option<i32>,
	pub updated_at: Option<i32>,
	#[cynic(rename = "type")]
	pub type_: Option<MediaType>,
	pub trending: Option<i32>,
	pub synonyms: Option<Vec<Option<String>>>,
	pub tags: Option<Vec<Option<MediaTag>>>,
	pub status: Option<MediaStatus>,
	pub source: Option<MediaSource>,
	pub site_url: Option<String>,
	pub season_year: Option<i32>,
	pub season_int: Option<i32>,
	pub season: Option<MediaSeason>,
	pub popularity: Option<i32>,
	pub mod_notes: Option<String>,
	pub mean_score: Option<i32>,
	pub is_licensed: Option<bool>,
	pub is_adult: Option<bool>,
	pub hashtag: Option<String>,
	pub genres: Option<Vec<Option<String>>>,
	pub favourites: Option<i32>,
	pub format: Option<MediaFormat>,
	pub episodes: Option<i32>,
	pub end_date: Option<FuzzyDate>,
	pub duration: Option<i32>,
	pub description: Option<String>,
	pub country_of_origin: Option<CountryCode>,
	pub chapters: Option<i32>,
	pub banner_image: Option<String>,
	pub average_score: Option<i32>,
	pub auto_create_forum_thread: Option<bool>,
	pub characters: Option<CharacterConnection>,
	pub staff: Option<StaffConnection>,
	pub start_date: Option<FuzzyDate>,
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
	pub id: i32,
	pub name: Option<StaffName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct StaffName {
	pub user_preferred: Option<String>,
	pub native: Option<String>,
	pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaTag {
	pub category: Option<String>,
	pub description: Option<String>,
	pub id: i32,
	pub is_adult: Option<bool>,
	pub is_general_spoiler: Option<bool>,
	pub is_media_spoiler: Option<bool>,
	pub name: String,
	pub rank: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaTitle {
	pub english: Option<String>,
	pub native: Option<String>,
	pub romaji: Option<String>,
	pub user_preferred: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct MediaCoverImage {
	pub extra_large: Option<String>,
	pub medium: Option<String>,
	pub large: Option<String>,
	pub color: Option<String>,
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
	pub role: Option<CharacterRole>,
	pub node: Option<Character>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct Character {
	pub id: i32,
	pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct CharacterName {
	pub user_preferred: Option<String>,
	pub native: Option<String>,
	pub full: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]

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

fn embed_title(title: &MediaTitle) -> String {
	let en = title.english.clone();

	let rj = title.romaji.clone();

	let en = en.unwrap_or_default();

	let rj = rj.unwrap_or_default();

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

fn embed_desc(media: &Media) -> String {
	let mut desc = media.description.clone().unwrap_or_default();

	desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);

	let length_diff = 4096 - desc.len() as i32;

	if length_diff <= 0 {
		desc = trim(desc, length_diff)
	}

	desc
}

fn get_genre(genres: &[Option<String>]) -> String {
	genres
		.iter()
		.map(|string| string.clone().unwrap_or_default())
		.take(5)
		.collect::<Vec<String>>()
		.join("\n")
}

fn get_tag(tags: &[Option<MediaTag>]) -> String {
	tags.iter()
		.map(|media_tag| {
			media_tag
				.clone()
				.unwrap_or(MediaTag {
					category: None,
					description: None,
					id: 0,
					is_adult: None,
					is_general_spoiler: None,
					is_media_spoiler: None,
					name: "".to_string(),
					rank: None,
				})
				.name
		})
		.take(5)
		.collect::<Vec<String>>()
		.join("\n")
}

fn get_url(media: &Media) -> String {
	media
		.site_url
		.clone()
		.unwrap_or("https://example.com".to_string())
}

pub fn get_banner(media: &Media) -> String {
	format!("https://img.anili.st/media/{}", media.id)
}

fn get_date(date: &FuzzyDate) -> String {
	let date_y = date.year.unwrap_or(0);

	let date_d = date.day.unwrap_or(0);

	let date_m = date.month.unwrap_or(0);

	if date_y == 0 && date_d == 0 && date_m == 0 {
		UNKNOWN.to_string()
	} else {
		let mut date_of_birth_string = String::new();

		let mut has_month: bool = false;

		let mut has_day: bool = false;

		if let Some(m) = date.month {
			date_of_birth_string.push_str(format!("{:02}", m).as_str());

			has_month = true
		}

		if let Some(d) = date.day {
			if has_month {
				date_of_birth_string.push('/')
			}

			date_of_birth_string.push_str(format!("{:02}", d).as_str());

			has_day = true
		}

		if let Some(y) = date.year {
			if has_day {
				date_of_birth_string.push('/')
			}

			date_of_birth_string.push_str(format!("{:04}", y).as_str());
		}

		date_of_birth_string
	}
}

fn get_staff(staff: Vec<Option<StaffEdge>>) -> String {
	let mut staff_text = String::new();

	// iterate over staff with index
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

		let native = name.native;

		let staff_name = user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(UNKNOWN.to_string())));

		let s_role = s.role.clone();

		let role = s_role.unwrap_or(UNKNOWN.to_string());

		staff_text.push_str(format!("{}: {}\n", staff_name.as_str(), role.as_str()).as_str());

		i += 1;
	}

	staff_text
}

fn get_character(character: Vec<Option<CharacterEdge>>) -> String {
	let mut character_text = String::new();

	// iterate over staff with index
	let mut i = 0;

	for s in character.into_iter() {
		if i > 4 {
			break;
		}

		let name = match s {
			Some(s) => {
				let node = match s.node {
					Some(n) => n,
					None => continue,
				};

				let name = match node.name {
					Some(n) => n,
					None => continue,
				};

				let full = name.full;

				let user_pref = name.user_preferred;

				let native = name.native;

				user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(UNKNOWN.to_string())))
			},
			None => UNKNOWN.to_string(),
		};

		character_text.push_str(name.as_str());
		character_text.push('\n');

		i += 1;
	}

	character_text
}

pub async fn media_content<'a>(
	ctx: &'a SerenityContext, command_interaction: &'a CommandInteraction, data: Media,
	db_config: DbConfig, bot_data: Arc<BotData>,
) -> Result<EmbedsContents> {
	let is_adult = data.is_adult.unwrap_or(true);

	if is_adult && !get_nsfw(command_interaction, ctx).await {
		return Err(anyhow!("This an adult media in a non adult channel"));
	}

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let connection = bot_data.db_connection.clone();

	let anime_song = AnimeSong::find()
		.filter(AnilistId.eq(data.id.to_string()))
		.all(&*connection.clone())
		.await?;

	let mut song_list = anime_song
		.into_iter()
		.map(|song| {
			let mut message = song.song_name;
			if song.audio != String::new() {
				message.push_str(format!(" | [mp3]({})", song.audio).as_str());
			}
			if song.hq != String::new() {
				message.push_str(format!(" | [mp4]({})", song.hq).as_str());
			} else if song.mq != String::new() {
				message.push_str(format!(" | [mp4]({})", song.mq).as_str());
			}
			message.push('\n');
			message
		})
		.collect::<String>();

	if song_list.len() > 1024 {
		song_list.truncate(1024);
		// check until only \n is the last char
		while !song_list.ends_with('\n') {
			song_list.pop(); // Remove the last character until it ends with '\n'
		}
	}

	let media_localised = load_localization_media(guild_id, db_config).await?;

	let mut fields = Vec::new();

	if !song_list.is_empty() {
		fields.push((media_localised.song, song_list, false));
	}

	let genres = data.genres.clone().unwrap_or_default();

	// take the first 5 non-optional genres
	let genres = genres
		.into_iter()
		.flatten()
		.take(5)
		.collect::<Vec<String>>();

	let tag = data.tags.clone().unwrap_or_default();

	let tag = tag
		.into_iter()
		.filter_map(|t| if let Some(t) = t { Some(t.name) } else { None })
		.take(5)
		.collect::<Vec<String>>();

	fields.push((media_localised.tag, tag.join(", "), true));

	fields.push((media_localised.genre, genres.join(", "), true));

	if let Some(staff) = data.staff.clone() {
		if let Some(edges) = staff.edges {
			let staffs = get_staff(edges);

			fields.push((media_localised.staffs, staffs, false));
		}
	}

	if let Some(characters) = data.characters.clone() {
		if let Some(edges) = characters.edges {
			let characters = get_character(edges);

			fields.push((media_localised.characters, characters, true));
		}
	}

	if let Some(format) = data.format {
		fields.push((media_localised.format, format.to_string(), true))
	}

	if let Some(source) = data.source {
		fields.push((media_localised.source, source.to_string(), true))
	}

	if let Some(start_date) = data.start_date.clone() {
		let mut start_date_str = String::new();

		if let Some(day) = start_date.day {
			start_date_str.push_str(format!("{}/", day).as_str());
		}

		if let Some(month) = start_date.month {
			start_date_str.push_str(format!("{}/", month).as_str());
		}

		if let Some(year) = start_date.year {
			start_date_str.push_str(year.to_string().as_str());
		}

		fields.push((media_localised.start_date, start_date_str, true));
	}

	if let Some(end_date) = data.end_date.clone() {
		let mut end_date_str = String::new();

		if let Some(day) = end_date.day {
			end_date_str.push_str(format!("{}/", day).as_str());
		}

		if let Some(month) = end_date.month {
			end_date_str.push_str(format!("{}/", month).as_str());
		}

		if let Some(year) = end_date.year {
			end_date_str.push_str(year.to_string().as_str());
		}

		fields.push((media_localised.end_date, end_date_str, true));
	}

	if let Some(favourites) = data.favourites {
		fields.push((media_localised.fav, favourites.to_string(), true))
	}

	match data.duration {
		Some(duration) => {
			fields.push((
				media_localised.duration,
				format!("{} {}", duration, media_localised.minutes),
				true,
			));
		},
		None => {
			if let Some(chapters) = data.chapters {
				fields.push((
					media_localised.duration,
					format!("{} {}", chapters, media_localised.chapter),
					true,
				));
			}
		},
	}

	let title = match data.title.clone() {
		Some(t) => t,
		None => return Err(anyhow!("No title")),
	};

	let mut embed_content = EmbedContent::new(embed_title(&title))
		.url(get_url(&data.clone()))
		.fields(fields)
		.images_url(get_banner(&data.clone()));

	if let Some(image) = data.cover_image {
		if let Some(extra_large) = image.extra_large {
			embed_content = embed_content.images_url(extra_large);
		}
	}

	let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

	Ok(embed_contents)
}
