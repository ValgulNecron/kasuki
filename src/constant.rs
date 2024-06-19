use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use moka::future::Cache;
use once_cell::sync::Lazy;
use ratatui::style::Color;
use serenity::all::{Colour, CurrentApplicationInfo};
use tokio::sync::RwLock;

use crate::grpc_server::command_list::{get_list_of_all_command, CommandItem};

/// Delay before a new thread is spawned.
/// Delay between ping updates.
pub const PING_UPDATE_DELAYS: u64 = 600;
/// Time before a server image is updated.
pub const TIME_BEFORE_SERVER_IMAGE: u64 = 600;
/// Time between server image updates.
pub const TIME_BETWEEN_SERVER_IMAGE_UPDATE: u64 = 1_800;
/// Time between user color updates.
pub const TIME_BETWEEN_USER_COLOR_UPDATE: u64 = 300;
/// Time between steam updates.
pub const TIME_BETWEEN_GAME_UPDATE: u64 = 86_400;
/// Time between cache updates.
pub const TIME_BETWEEN_CACHE_UPDATE: u64 = 259_200;
/// Max capacity for the cache.
pub const CACHE_MAX_CAPACITY: u64 = 100_000;
/// Anilist cache for the bot.
pub static mut ANILIST_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build()
});
/// vndb cache for the bot.
pub static mut VNDB_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build()
});
/// Limit for autocomplete count.
pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 25;
/// Limit for pass count.
pub const PASS_LIMIT: u32 = 10;
/// Limit for member list.
pub const MEMBER_LIST_LIMIT: u64 = 10;
/// Limit for activity list.
pub const ACTIVITY_LIST_LIMIT: u64 = 1;
/// Path to the data SQLite database.
pub const SQLITE_DB_PATH: &str = "db/data.db";
pub const COMMAND_USE_PATH: &str = "db/command_use.json";
pub const RANDOM_STATS_PATH: &str = "db/random_stats.json";
/*
App embed color.
 */
/// Color for the app embed.
pub const COLOR: Colour = Colour::FABLED_PINK;
pub const TUI_FG_COLOR: Color = Color::Rgb(250, 177, 237);

/// Log level for other crates.
pub const OTHER_CRATE_LEVEL: &str = "warn";
/// Default string value.
pub const UNKNOWN: &str = "Unknown";

/// Map of language codes to language names.
pub const LANG_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let languages = [
        ("en", "english"),
        ("fr", "french"),
        ("de", "german"),
        ("ja", "japanese"),
    ];
    languages.iter().cloned().collect()
});

/// Map of app names to their respective IDs.
pub static mut APPS: Lazy<HashMap<String, u128>> = Lazy::new(HashMap::new);

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

/// User blacklist for the server image.
pub static mut USER_BLACKLIST_SERVER_IMAGE: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| {
    let user_ids: Vec<String> = Vec::new();
    Arc::from(RwLock::from(user_ids))
});

/// The bot's information.
pub static mut BOT_INFO: Option<CurrentApplicationInfo> = None;
/// Vec of all available bot commands.
pub const BOT_COMMANDS: Lazy<Vec<CommandItem>> = Lazy::new(get_list_of_all_command);
/// Used library.
pub const LIBRARY: &str = "serenity";
pub static mut CONFIG: Lazy<Config> = Lazy::new(Config::default);
