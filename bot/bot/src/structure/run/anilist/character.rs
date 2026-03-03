use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::trimer::trim;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use sea_orm::DatabaseConnection;
use serenity::all::CommandInteraction;
use shared::localization::{get_language_identifier, USABLE_LOCALES};
use std::sync::Arc;
use tracing::log::trace;

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
pub async fn character_content<'a>(
	command_interaction: CommandInteraction, character: Character,
	db_connection: Arc<DatabaseConnection>,
) -> Result<EmbedsContents<'a>> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	trace!("{:#?}", guild_id);

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let date_of_birth_data = character.date_of_birth.clone();

	let mut fields = Vec::new();

	if let Some(date_of_birth_data) = date_of_birth_data {
		let mut has_month: bool = false;

		let mut has_day: bool = false;

		let mut date_of_birth_string = String::new();

		if let Some(m) = date_of_birth_data.month {
			date_of_birth_string.push_str(format!("{:02}", m).as_str());

			has_month = true
		}

		if let Some(d) = date_of_birth_data.day {
			if has_month {
				date_of_birth_string.push('/')
			}

			date_of_birth_string.push_str(format!("{:02}", d).as_str());

			has_day = true
		}

		if let Some(y) = date_of_birth_data.year {
			if has_day {
				date_of_birth_string.push('/')
			}

			date_of_birth_string.push_str(format!("{:04}", y).as_str());
		}

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-date_of_birth"),
			date_of_birth_string,
			true,
		));
	}

	let gender = character.gender.clone();

	if let Some(gender) = gender {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-gender"),
			gender,
			true,
		));
	}

	let age = character.age.clone();

	if let Some(age) = age {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-age"),
			age,
			true,
		));
	}

	let favourites = character.favourites;

	if let Some(favourites) = favourites {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-fav"),
			favourites.to_string(),
			true,
		));
	}

	let blood_type = character.blood_type.clone();

	if let Some(blood_type) = blood_type {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-blood_type"),
			blood_type,
			true,
		));
	}

	let mut desc = character.description.unwrap_or_default();

	desc = convert_anilist_flavored_to_discord_flavored_markdown(desc);

	let length_diff = 4096 - desc.len() as i32;

	if length_diff <= 0 {
		desc = trim(desc, length_diff)
	}

	let name = match character.name.clone() {
		Some(name) => name,
		None => return Err(anyhow!("No name found".to_string())),
	};

	let native = name.native.unwrap_or_default();

	let user_pref = name.user_preferred.unwrap_or_default();

	let character_name = format!("{}/{}", user_pref, native);

	let mut embeds_content = EmbedContent::new(character_name)
		.description(desc)
		.url(character.site_url.unwrap_or_default())
		.fields(fields);

	if let Some(image) = character.image {
		if let Some(large) = image.large {
			embeds_content = embeds_content.images_url(large);
		}
	}

	let embeds_contents = EmbedsContents::new(vec![embeds_content]);

	Ok(embeds_contents)
}
