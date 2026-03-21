use crate::helper::get_guild_lang::get_guild_language;
use anyhow::Result;
pub use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::static_loader;
pub use fluent_templates::Loader;
use sea_orm::DatabaseConnection;
use std::str::FromStr;
use std::sync::Arc;
pub use unic_langid::LanguageIdentifier;

static_loader! {
	pub static USABLE_LOCALES = {
		locales: "../translation",
		fallback_language: "en-US",
	};
}

pub fn load_locales() -> Result<()> {
	Ok(())
}

/// Returns the list of available locales baked in at compile time by `static_loader!`.
pub fn available_locales() -> Vec<String> {
	let mut locales: Vec<String> = USABLE_LOCALES.locales().map(|l| l.to_string()).collect();
	locales.sort();
	locales
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

#[cfg(test)]
mod tests {
	use std::collections::{HashMap, HashSet};
	use std::fs;
	use std::path::Path;
	use walkdir::WalkDir;

	const FALLBACK_LOCALE: &str = "en-US";
	const EXPECTED_LOCALES: &[&str] = &["en-US", "ja", "de", "fr"];

	fn translation_dir() -> std::path::PathBuf {
		Path::new(env!("CARGO_MANIFEST_DIR")).join("../translation")
	}

	fn collect_ftl_files(dir: &Path) -> Vec<std::path::PathBuf> {
		WalkDir::new(dir)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
			.map(|e| e.into_path())
			.collect()
	}

	fn parse_message_ids(content: &str) -> Vec<String> {
		match fluent_syntax::parser::parse(content) {
			Ok(resource) => resource
				.body
				.iter()
				.filter_map(|entry| {
					if let fluent_syntax::ast::Entry::Message(msg) = entry {
						Some(msg.id.name.to_string())
					} else {
						None
					}
				})
				.collect(),
			Err((resource, _)) => resource
				.body
				.iter()
				.filter_map(|entry| {
					if let fluent_syntax::ast::Entry::Message(msg) = entry {
						Some(msg.id.name.to_string())
					} else {
						None
					}
				})
				.collect(),
		}
	}

	#[test]
	fn all_ftl_files_are_valid_syntax() {
		let dir = translation_dir();
		let files = collect_ftl_files(&dir);
		assert!(
			!files.is_empty(),
			"No .ftl files found in {}",
			dir.display()
		);

		let mut errors = Vec::new();

		for path in &files {
			let content = fs::read_to_string(path)
				.unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

			if let Err((_resource, parse_errors)) = fluent_syntax::parser::parse(content.as_str()) {
				for err in parse_errors {
					errors.push(format!(
						"{}:{:?} - {:?}",
						path.strip_prefix(&dir).unwrap_or(path).display(),
						err.pos,
						err.kind
					));
				}
			}
		}

		if !errors.is_empty() {
			panic!(
				"Found {} Fluent parsing error(s):\n{}",
				errors.len(),
				errors.join("\n")
			);
		}
	}

	#[test]
	fn all_expected_locales_exist() {
		let dir = translation_dir();
		for locale in EXPECTED_LOCALES {
			let locale_dir = dir.join(locale);
			assert!(
				locale_dir.is_dir(),
				"Missing locale directory: {}",
				locale_dir.display()
			);
		}
	}

	#[test]
	fn all_locales_have_same_ftl_files() {
		let dir = translation_dir();
		let mut locale_files: HashMap<String, HashSet<String>> = HashMap::new();

		for locale in EXPECTED_LOCALES {
			let locale_dir = dir.join(locale);
			let files: HashSet<String> = fs::read_dir(&locale_dir)
				.unwrap_or_else(|e| panic!("Failed to read {}: {}", locale_dir.display(), e))
				.filter_map(|e| e.ok())
				.filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
				.map(|e| e.file_name().to_string_lossy().to_string())
				.collect();
			locale_files.insert(locale.to_string(), files);
		}

		let fallback_files = &locale_files[FALLBACK_LOCALE];
		let mut errors = Vec::new();

		for locale in EXPECTED_LOCALES {
			if *locale == FALLBACK_LOCALE {
				continue;
			}
			let files = &locale_files[*locale];

			for missing in fallback_files.difference(files) {
				errors.push(format!("{} is missing file: {}", locale, missing));
			}

			for extra in files.difference(fallback_files) {
				errors.push(format!(
					"{} has extra file not in {}: {}",
					locale, FALLBACK_LOCALE, extra
				));
			}
		}

		if !errors.is_empty() {
			panic!("Locale file mismatches:\n{}", errors.join("\n"));
		}
	}

	#[test]
	fn all_locales_have_same_message_ids() {
		let dir = translation_dir();
		let fallback_dir = dir.join(FALLBACK_LOCALE);

		let fallback_ftl_files: Vec<String> = fs::read_dir(&fallback_dir)
			.unwrap()
			.filter_map(|e| e.ok())
			.filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
			.map(|e| e.file_name().to_string_lossy().to_string())
			.collect();

		let mut errors = Vec::new();

		for ftl_file in &fallback_ftl_files {
			let fallback_content = fs::read_to_string(fallback_dir.join(ftl_file)).unwrap();
			let fallback_ids: HashSet<String> =
				parse_message_ids(&fallback_content).into_iter().collect();

			for locale in EXPECTED_LOCALES {
				if *locale == FALLBACK_LOCALE {
					continue;
				}
				let locale_path = dir.join(locale).join(ftl_file);
				let locale_content = match fs::read_to_string(&locale_path) {
					Ok(c) => c,
					Err(_) => continue, // Missing file is caught by another test
				};
				let locale_ids: HashSet<String> =
					parse_message_ids(&locale_content).into_iter().collect();

				for missing in fallback_ids.difference(&locale_ids) {
					errors.push(format!(
						"{}/{} is missing message: {}",
						locale, ftl_file, missing
					));
				}

				for extra in locale_ids.difference(&fallback_ids) {
					errors.push(format!(
						"{}/{} has extra message not in {}: {}",
						locale, ftl_file, FALLBACK_LOCALE, extra
					));
				}
			}
		}

		if !errors.is_empty() {
			panic!("Message ID mismatches:\n{}", errors.join("\n"));
		}
	}

	#[test]
	fn no_empty_ftl_files() {
		let dir = translation_dir();
		let mut errors = Vec::new();

		for path in collect_ftl_files(&dir) {
			let content = fs::read_to_string(&path).unwrap();
			if content.trim().is_empty() {
				errors.push(format!(
					"{} is empty",
					path.strip_prefix(&dir).unwrap_or(&path).display()
				));
			}
		}

		if !errors.is_empty() {
			panic!("Found empty .ftl files:\n{}", errors.join("\n"));
		}
	}

	#[test]
	fn no_ftl_messages_have_empty_values() {
		let dir = translation_dir();
		let mut errors = Vec::new();

		for path in collect_ftl_files(&dir) {
			let content = fs::read_to_string(&path).unwrap();
			let resource = match fluent_syntax::parser::parse(content.as_str()) {
				Ok(r) => r,
				Err((r, _)) => r,
			};

			for entry in &resource.body {
				if let fluent_syntax::ast::Entry::Message(msg) = entry {
					if msg.value.is_none() && msg.attributes.is_empty() {
						errors.push(format!(
							"{}: message '{}' has no value and no attributes",
							path.strip_prefix(&dir).unwrap_or(&path).display(),
							msg.id.name
						));
					}
				}
			}
		}

		if !errors.is_empty() {
			panic!(
				"Found {} message(s) with no value:\n{}",
				errors.len(),
				errors.join("\n")
			);
		}
	}
}
