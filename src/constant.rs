use std::collections::HashMap;
use std::env;

use once_cell::sync::Lazy;
use serenity::all::Colour;

/// The activity name of the bot, fetched from the environment variable "BOT_ACTIVITY".
/// If the environment variable is not set, it defaults to "Let you get info from anilist.".
pub static ACTIVITY_NAME: Lazy<String> = Lazy::new(|| {
    env::var("BOT_ACTIVITY").unwrap_or_else(|_| "Let you get info from anilist.".to_string())
});

/// The version of the application, fetched from the environment variable "CARGO_PKG_VERSION".
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/*
All delays in seconds.
 */
/// Delay before a new thread is spawned.
/// Delay between ping updates.
pub const PING_UPDATE_DELAYS: u64 = 600;
/// Time before a server image is updated.
pub const TIME_BEFORE_SERVER_IMAGE: u64 = 600;
/// Time between server image updates.
pub const TIME_BETWEEN_SERVER_IMAGE_UPDATE: u64 = 86_400;
/// Time between user color updates.
pub const TIME_BETWEEN_USER_COLOR_UPDATE: u64 = 1_800;
/// Time between steam updates.
pub const TIME_BETWEEN_GAME_UPDATE: u64 = 86_400;
/// Time between cache updates.
pub const TIME_BETWEEN_CACHE_UPDATE: u64 = 259_200;

/*
Limits.
 */
/// Limit for autocomplete count.
pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 20;
/// Limit for pass count.
pub const PASS_LIMIT: u32 = 10;
/// Limit for member list.
pub const MEMBER_LIST_LIMIT: u64 = 10;
/// Limit for activity list.
pub const ACTIVITY_LIST_LIMIT: u64 = 10;
/*
Database paths.
 */
/// Path to the data SQLite database.
pub const DATA_SQLITE_DB: &str = "./data.db";
/// Path to the cache SQLite database.
pub const CACHE_SQLITE_DB: &str = "./cache.db";

/*
App embed color.
 */
/// Color for the app embed.
pub const COLOR: Colour = Colour::FABLED_PINK;

/*
Other crate log level. Because serenity uses info to obviously trace things like heartbeat.
 */
/// Log level for other crates.
pub const OTHER_CRATE_LEVEL: &str = "warn";

/*
Default value.
 */
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

/// Map of app names to their respective IDs.
pub static mut APPS: Lazy<HashMap<String, u128>> = Lazy::new(HashMap::new);

/// Path to the logs.
pub const LOGS_PATH: &str = "./logs";
/// Prefix for the logs.
pub const LOGS_PREFIX: &str = "kasuki_";
/// Suffix for the logs
pub const LOGS_SUFFIX: &str = ".log";
/// Maximum log retention days.
pub static MAX_LOG_RETENTION_DAYS: Lazy<u64> = Lazy::new(|| {
    env::var("MAX_LOG_RETENTION_DAYS")
        .unwrap_or("7".to_string())
        .parse()
        .unwrap_or(7)
});

/// Guard for the non-blocking appender.
pub static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

/// Path to the server image.
pub const SERVER_IMAGE_PATH: &str = "server_image";

/// Default string value.
pub const DEFAULT_STRING: &String = &String::new();

/*
AI stuff
 */

/*
Image
 */
/// Base URL for the AI image API.
pub static mut IMAGE_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}images/generations",
        env::var("AI_IMAGE_API_BASE_URL").unwrap_or_else(
            |_| env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});

/// Token for the AI image API.
pub static mut IMAGE_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_IMAGE_API_TOKEN").unwrap_or_else(|_| env::var("AI_API_TOKEN").unwrap_or_default())
});

/// Models for the AI image API.
pub static mut IMAGE_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_IMAGE_GENERATION_MODELS").unwrap_or_else(|_| "dall-e-3".to_string()));

/*
Chat and translation
 */
/// Base URL for the AI chat API.
pub static CHAT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}chat/completions",
        env::var("AI_CHAT_API_BASE_URL").unwrap_or_else(
            |_| env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});

/// Token for the AI chat API.
pub static CHAT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_CHAT_API_TOKEN").unwrap_or_else(|_| env::var("AI_API_TOKEN").unwrap_or_default())
});

/// Models for the AI chat API.
pub static CHAT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_CHAT_MODEL").unwrap_or("gpt-3.5-turbo".to_string()));

/*
Transcription
 */
/// Base URL for the AI transcription API.
pub static TRANSCRIPT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}audio/",
        env::var("AI_TRANSCRIPT_BASE_URL")
            .or_else(|_| env::var("AI_API_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1/".to_string())
    )
});

/// Token for the AI transcription API.
pub static TRANSCRIPT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_TRANSCRIPT_API_TOKEN")
        .or_else(|_| env::var("AI_API_TOKEN"))
        .unwrap_or_default()
});

/// Models for the AI transcription API.
pub static TRANSCRIPT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_TRANSCRIPT_MODELS").unwrap_or_else(|_| String::from("whisper-1")));

/*
Web server
*/
/// Flag to enable or disable the web server.
pub static GRPC_IS_ON: Lazy<bool> =
    Lazy::new(|| env::var("GRPC_IS_ON").unwrap_or_else(|_| "false".to_string()) == "true");

/// Port for the web server.
pub static GRPC_SERVER_PORT: Lazy<String> =
    Lazy::new(|| env::var("GRPC_SERVER_PORT").unwrap_or_else(|_| "8080  ".to_string()));
