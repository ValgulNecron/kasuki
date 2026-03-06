use std::collections::HashMap;
use std::env;

use once_cell::sync::Lazy;
use serenity::all::Colour;

pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 25;

/*
App embed color.
 */

/// Color for the app embed.

pub const COLOR: Colour = Colour::FABLED_PINK;

/// Log level for other crates.

pub const OTHER_CRATE_LEVEL: &str = "warn";

/// Default string value.

pub const UNKNOWN: &str = "Unknown";

/// Map of language codes to language names.

pub static LANG_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
	let languages = [
		("en", "english"),
		("fr", "french"),
		("de", "german"),
		("ja", "japanese"),
	];

	languages.iter().cloned().collect()
});

/// Path to the logs.

pub const LOGS_PATH: &str = "./logs";

/// Prefix for the logs.

pub const LOGS_PREFIX: &str = "kasuki_";

/// Suffix for the logs

pub const LOGS_SUFFIX: &str = "log";

/// Default string value used as a fallback when a command option is not provided.
pub const DEFAULT_STRING: &str = "";

/// The version of the application, fetched from the environment variable "CARGO_PKG_VERSION".

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vec of all available bot commands.

/// Used library.

pub const LIBRARY: &str = "serenity";

pub const ACTIVITY_LIST_LIMIT: u64 = 10;
pub const MEMBER_LIST_LIMIT: u16 = 10;
