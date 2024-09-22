use std::collections::HashMap;
use std::env;

use once_cell::sync::Lazy;
use serenity::all::Colour;

use crate::api::grpc_server::command_list::{get_list_of_all_command, CommandItem};

/// Delay before a new thread is spawned.
/// Delay between ping updates.

pub const TIME_BETWEEN_PING_UPDATE: u64 = 600;

/// Time before a server image is updated.

pub const TIME_BEFORE_SERVER_IMAGE: u64 = 1_800;

/// Time between server image updates.

pub const TIME_BETWEEN_SERVER_IMAGE_UPDATE: u64 = 1_800;

/// Time between user color updates.

pub const TIME_BETWEEN_USER_COLOR_UPDATE: u64 = 300;

/// Time between steam updates.

pub const TIME_BETWEEN_GAME_UPDATE: u64 = 86_400;

/// Time between cache updates.

pub const TIME_BETWEEN_CACHE_UPDATE: u64 = 259_200;

/// Time between bot info update

pub const TIME_BETWEEN_BOT_INFO: u64 = 1_800;

/// time between blacklisted user update

pub const TIME_BETWEEN_BLACKLISTED_USER_UPDATE: u64 = 3600;

/// time between activity check.

pub const TIME_BETWEEN_ACTIVITY_CHECK: u64 = 1;

/// time between random stats update.

pub const TIME_BETWEEN_RANDOM_STATS_UPDATE: u64 = 86_400;

/// Max capacity for the cache.

pub const CACHE_MAX_CAPACITY: u64 = 100_000;

/// Limit for autocomplete count.

pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 25;

/// Limit for pass count.

pub const PASS_LIMIT: u32 = 10;

pub const THREAD_POOL_SIZE: usize = 25;

/// Limit for member list.

pub const MEMBER_LIST_LIMIT: u64 = 10;

/// Limit for activity list.

pub const ACTIVITY_LIST_LIMIT: u64 = 10;

/// Path to the data SQLite database.

pub const COMMAND_USE_PATH: &str = "db/command_use.json";

pub const RANDOM_STATS_PATH: &str = "db/random_stats.json";

pub const NEW_MEMBER_PATH: &str = "db/new_member.json";

pub const NEW_MEMBER_IMAGE_PATH: &str = "new_member_image/";

/*
App embed color.
 */

/// Color for the app embed.

pub const COLOR: Colour = Colour::FABLED_PINK;

pub const HEX_COLOR: &str = "#FAB1ED";

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

/// Guard for the non-blocking appender.

pub static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

/// Default string value.

pub const DEFAULT_STRING: &String = &String::new();

/// The version of the application, fetched from the environment variable "CARGO_PKG_VERSION".

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Vec of all available bot commands.

pub static BOT_COMMANDS: Lazy<Vec<CommandItem>> = Lazy::new(get_list_of_all_command);

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

pub const ONE_HOUR: u64 = 60 * 60;

pub const ONE_DAY: u64 = 24 * ONE_HOUR;
