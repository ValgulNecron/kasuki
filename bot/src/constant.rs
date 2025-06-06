use std::collections::HashMap;
use std::env;

use once_cell::sync::Lazy;
use serenity::all::Colour;

/// Time between cache updates.
pub const TIME_BETWEEN_CACHE_UPDATE: u64 = 259_200;

/// Max capacity for the cache.
pub const CACHE_MAX_CAPACITY: u64 = 100_000;

/// Limit for autocomplete count.
pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 25;

pub const THREAD_POOL_SIZE: usize = 25;

/// Limit for member list.
pub const MEMBER_LIST_LIMIT: u16 = 10;

/// Limit for activity list.
pub const ACTIVITY_LIST_LIMIT: u64 = 10;

/// Path to the data SQLite database.
pub const COMMAND_USE_PATH: &str = "db/command_use.json";

pub const RANDOM_STATS_PATH: &str = "db/random_stats.json";

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
	let languages = [("en", "english"), ("fr", "french"), ("de", "german"), ("ja", "japanese")];

	languages.iter().cloned().collect()
});

/// Path to the logs.

pub const LOGS_PATH: &str = "./logs";

/// Prefix for the logs.

pub const LOGS_PREFIX: &str = "kasuki_";

/// Suffix for the logs

pub const LOGS_SUFFIX: &str = "log";

/// Default string value.

pub const DEFAULT_STRING: &String = &String::new();

/// The version of the application, fetched from the environment variable "CARGO_PKG_VERSION".

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vec of all available bot commands.

/// Used library.

pub const LIBRARY: &str = "serenity";

pub const MAX_FREE_AI_IMAGES: usize = 5;

pub const PAID_IMAGE_MULTIPLIER: f64 = 4.0;

pub const MAX_FREE_AI_QUESTIONS: usize = 5;

pub const PAID_QUESTION_MULTIPLIER: f64 = 5.0;

pub const MAX_FREE_AI_TRANSLATIONS: usize = 5;

pub const PAID_TRANSLATION_MULTIPLIER: f64 = 5.0;

pub const MAX_FREE_AI_TRANSCRIPTS: usize = 5;

pub const PAID_TRANSCRIPT_MULTIPLIER: f64 = 5.0;
