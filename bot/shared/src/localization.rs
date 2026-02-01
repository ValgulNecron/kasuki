use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use anyhow::{Context, Result};
use fluent_templates::ArcLoader;
pub use fluent_templates::Loader;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
pub use unic_langid::LanguageIdentifier;

static LOCALES_INNER: OnceLock<ArcLoader> = OnceLock::new();

pub struct UsableLocales;

impl std::ops::Deref for UsableLocales {
	type Target = ArcLoader;

	fn deref(&self) -> &Self::Target {
		LOCALES_INNER
			.get()
			.expect("Locales not initialized. Call shared::localization::load_locales() first.")
	}
}

pub static USABLE_LOCALES: UsableLocales = UsableLocales;

pub fn load_locales() -> Result<()> {
	if LOCALES_INNER.get().is_some() {
		return Ok(());
	}

	let loader = ArcLoader::builder("translation", "en-US".parse()?)
		.build()
		.map_err(|e| anyhow::anyhow!(e.to_string()))
		.context("Failed to load locales")?;

	let _ = LOCALES_INNER.set(loader);

	Ok(())
}

pub async fn get_language_identifier(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> LanguageIdentifier {
	let lang_choice = get_guild_language(guild_id, db_connection).await;
	let lang_code = match lang_choice.as_str() {
		"jp" => "ja",
		"en" => "en-US",
		other => other,
	};
	LanguageIdentifier::from_str(lang_code)
		.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap())
}

pub async fn load_localization<'a, T: serde::Deserialize<'a> + Clone>(
	guild_id: String, path: &str, db_connection: Arc<DatabaseConnection>,
) -> Result<T> {
	let json_content =
		read_file_as_string(path).context(format!("Failed to read file: {}", path))?;

	let json: &'a str = Box::leak(json_content.into_boxed_str());

	let json_data: HashMap<String, T> =
		serde_json::from_str(json).context("Failed to parse JSON data")?;

	let mut lang_choice = get_guild_language(guild_id, db_connection).await;

	if lang_choice == "en-US" {
		lang_choice = "en".to_string();
	} else if lang_choice == "ja" {
		lang_choice = "jp".to_string();
	}
	

	Ok(json_data.get(lang_choice.as_str()).cloned().unwrap_or(
		json_data
			.get("en")
			.cloned()
			.context("Failed to get English localization")?,
	))
}
