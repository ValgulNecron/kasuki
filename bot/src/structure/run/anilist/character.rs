use crate::config::DbConfig;
use crate::constant::COLOR;
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::trimer::trim;
use crate::structure::message::anilist_user::character::load_localization_character;
use anyhow::{anyhow, Result};
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateEmbed, CreateInteractionResponse,
	CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;
use crate::command::command_trait::{EmbedContent, EmbedType};

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
	pub id: i32,
	pub image: Option<CharacterImage>,
	pub mod_notes: Option<String>,
	pub name: Option<CharacterName>,
	pub site_url: Option<String>,
	pub updated_at: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct CharacterName {
	pub user_preferred: Option<String>,
	pub native: Option<String>,
	pub full: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct CharacterImage {
	pub medium: Option<String>,
	pub large: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]

pub struct FuzzyDate {
	pub month: Option<i32>,
	pub year: Option<i32>,
	pub day: Option<i32>,
}
pub async fn character_content(
	command_interaction: &CommandInteraction, character: Character,
	db_config: DbConfig,
) -> Result<EmbedContent> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	trace!("{:#?}", guild_id);

	let character_localised = load_localization_character(guild_id, db_config).await?;

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
			character_localised.date_of_birth,
			date_of_birth_string,
			true,
		));
	}

	let gender = character.gender.clone();

	if let Some(gender) = gender {
		fields.push((character_localised.gender, gender, true));
	}

	let age = character.age.clone();

	if let Some(age) = age {
		fields.push((character_localised.age, age, true));
	}

	let favourites = character.favourites;

	if let Some(favourites) = favourites {
		fields.push((character_localised.fav, favourites.to_string(), true));
	}

	let blood_type = character.blood_type.clone();

	if let Some(blood_type) = blood_type {
		fields.push((character_localised.blood_type, blood_type, true));
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

	let mut content = EmbedContent {
		title: character_name,
		description: desc,
		thumbnail: None,
		url: Some(character.site_url.unwrap_or_default()),
		command_type: EmbedType::First,
		colour: None,
		fields,
		images: None,
		action_row: None,
		images_url: None,
	};

	if let Some(image) = character.image {
		if let Some(large) = image.large {
			content.images_url = Some(large);
		}
	}

	Ok(content)
}
