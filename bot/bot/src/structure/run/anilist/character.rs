pub use shared::anilist::character::*;

use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
use crate::helper::trimer::trim;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use shared::localization::USABLE_LOCALES;
use unic_langid::LanguageIdentifier;

pub async fn character_content<'a>(
	character: Character, lang_id: &LanguageIdentifier,
) -> Result<EmbedsContents<'a>> {
	let mut fields = Vec::new();

	if let Some(date_of_birth_data) = &character.date_of_birth {
		let date_of_birth_string = format_fuzzy_date(date_of_birth_data);

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-date_of_birth"),
			date_of_birth_string,
			true,
		));
	}

	if let Some(gender) = character.gender {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_character-gender"),
			gender,
			true,
		));
	}

	if let Some(age) = character.age {
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

	if let Some(blood_type) = character.blood_type {
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

	let name = character.name.as_ref().ok_or_else(|| anyhow!("No name found"))?;

	let character_name = format_character_name(name);

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

#[cfg(test)]
mod tests {
	use super::*;

	fn make_character(
		name: Option<CharacterName>, date_of_birth: Option<FuzzyDate>,
		gender: Option<&str>, age: Option<&str>, favourites: Option<i32>,
		blood_type: Option<&str>, description: Option<&str>,
	) -> Character {
		Character {
			age: age.map(|s| s.to_string()),
			blood_type: blood_type.map(|s| s.to_string()),
			date_of_birth,
			description: description.map(|s| s.to_string()),
			favourites,
			gender: gender.map(|s| s.to_string()),
			image: None,
			name,
			site_url: Some("https://anilist.co/character/1".to_string()),
		}
	}

	fn default_name() -> Option<CharacterName> {
		Some(CharacterName {
			user_preferred: Some("Test".to_string()),
			native: Some("\u{30c6}\u{30b9}\u{30c8}".to_string()),
		})
	}

	fn lang_id() -> LanguageIdentifier {
		"en-US".parse::<LanguageIdentifier>().unwrap()
	}

	// --- format_fuzzy_date tests ---

	#[test]
	fn date_month_day_year() {
		let date = FuzzyDate {
			month: Some(3),
			day: Some(25),
			year: Some(1990),
		};
		assert_eq!(format_fuzzy_date(&date), "03/25/1990");
	}

	#[test]
	fn date_month_only() {
		let date = FuzzyDate {
			month: Some(12),
			day: None,
			year: None,
		};
		assert_eq!(format_fuzzy_date(&date), "12");
	}

	#[test]
	fn date_month_day() {
		let date = FuzzyDate {
			month: Some(7),
			day: Some(4),
			year: None,
		};
		assert_eq!(format_fuzzy_date(&date), "07/04");
	}

	#[test]
	fn date_year_only() {
		let date = FuzzyDate {
			month: None,
			day: None,
			year: Some(2000),
		};
		assert_eq!(format_fuzzy_date(&date), "2000");
	}

	#[test]
	fn date_all_none() {
		let date = FuzzyDate {
			month: None,
			day: None,
			year: None,
		};
		assert_eq!(format_fuzzy_date(&date), "");
	}

	// --- format_character_name tests ---

	#[test]
	fn name_both_present() {
		let name = CharacterName {
			user_preferred: Some("Saber".to_string()),
			native: Some("\u{30bb}\u{30a4}\u{30d0}\u{30fc}".to_string()),
		};
		assert_eq!(
			format_character_name(&name),
			"Saber/\u{30bb}\u{30a4}\u{30d0}\u{30fc}"
		);
	}

	#[test]
	fn name_user_preferred_only() {
		let name = CharacterName {
			user_preferred: Some("Saber".to_string()),
			native: None,
		};
		assert_eq!(format_character_name(&name), "Saber/");
	}

	#[test]
	fn name_native_only() {
		let name = CharacterName {
			user_preferred: None,
			native: Some("\u{30bb}\u{30a4}\u{30d0}\u{30fc}".to_string()),
		};
		assert_eq!(
			format_character_name(&name),
			"/\u{30bb}\u{30a4}\u{30d0}\u{30fc}"
		);
	}

	#[test]
	fn name_both_none() {
		let name = CharacterName {
			user_preferred: None,
			native: None,
		};
		assert_eq!(format_character_name(&name), "/");
	}

	// --- character_content tests ---

	#[tokio::test]
	async fn content_name_none_returns_error() {
		let character = make_character(None, None, None, None, None, None, None);
		let result = character_content(character, &lang_id()).await;
		assert!(result.is_err());
	}

	#[tokio::test]
	async fn content_date_of_birth_field_present() {
		let character = make_character(
			default_name(),
			Some(FuzzyDate {
				month: Some(1),
				day: Some(15),
				year: Some(2001),
			}),
			None,
			None,
			None,
			None,
			None,
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let dob_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Date of Birth"))
			.expect("Date of Birth field should exist");
		assert_eq!(dob_field.1, "01/15/2001");
	}

	#[tokio::test]
	async fn content_no_date_of_birth_no_field() {
		let character =
			make_character(default_name(), None, None, None, None, None, None);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let dob_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Date of Birth"));
		assert!(dob_field.is_none());
	}

	#[tokio::test]
	async fn content_gender_field_present() {
		let character = make_character(
			default_name(),
			None,
			Some("Female"),
			None,
			None,
			None,
			None,
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let gender_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Gender"))
			.expect("Gender field should exist");
		assert_eq!(gender_field.1, "Female");
	}

	#[tokio::test]
	async fn content_age_field_present() {
		let character = make_character(
			default_name(),
			None,
			None,
			Some("17"),
			None,
			None,
			None,
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let age_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Age"))
			.expect("Age field should exist");
		assert_eq!(age_field.1, "17");
	}

	#[tokio::test]
	async fn content_favourites_field_present() {
		let character = make_character(
			default_name(),
			None,
			None,
			None,
			Some(9001),
			None,
			None,
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let fav_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Favorites"))
			.expect("Favorites field should exist");
		assert_eq!(fav_field.1, "9001");
	}

	#[tokio::test]
	async fn content_blood_type_field_present() {
		let character = make_character(
			default_name(),
			None,
			None,
			None,
			None,
			Some("AB"),
			None,
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let blood_field = embed
			.fields
			.iter()
			.find(|(name, _, _)| name.contains("Blood Type"))
			.expect("Blood Type field should exist");
		assert_eq!(blood_field.1, "AB");
	}

	#[tokio::test]
	async fn content_long_description_gets_trimmed() {
		let long_desc = "a".repeat(5000);
		let character = make_character(
			default_name(),
			None,
			None,
			None,
			None,
			None,
			Some(&long_desc),
		);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		let desc = embed.description.as_ref().unwrap();
		assert!(
			desc.len() <= 4096,
			"Description should be trimmed to 4096 chars or fewer, got {}",
			desc.len()
		);
		assert!(desc.ends_with("..."));
	}

	#[tokio::test]
	async fn content_title_uses_formatted_name() {
		let character =
			make_character(default_name(), None, None, None, None, None, None);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		assert_eq!(
			embed.title,
			"Test/\u{30c6}\u{30b9}\u{30c8}"
		);
	}

	#[tokio::test]
	async fn content_no_optional_fields_produces_empty_fields() {
		let character =
			make_character(default_name(), None, None, None, None, None, None);
		let result = character_content(character, &lang_id()).await.unwrap();
		let embed = &result.embed_contents[0];
		assert!(embed.fields.is_empty());
	}
}
