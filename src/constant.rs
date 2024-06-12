use moka::future::Cache;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use ratatui::style::Color;
use serenity::all::{Colour, CurrentApplicationInfo};
use tokio::sync::RwLock;

use crate::grpc_server::command_list::{get_list_of_all_command, CommandItem};

pub const DISCORD_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("DISCORD_TOKEN").expect("Expected a token in the environment"));

/// The activity name of the bot, fetched from the environment variable "BOT_ACTIVITY".
/// If the environment variable is not set, it defaults to "Let you get info from anilist_user.".
pub const ACTIVITY_NAME: Lazy<String> = Lazy::new(|| {
    env::var("BOT_ACTIVITY").unwrap_or_else(|_| "Let you get info from anilist_user.".to_string())
});

/*
All delays in seconds.
 */
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
/// cache for the bot.
pub static mut CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
        .max_capacity(CACHE_MAX_CAPACITY)
        .build()
});
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
/*
App embed color.
 */
/// Color for the app embed.
pub const COLOR: Colour = Colour::FABLED_PINK;
pub const TUI_FG_COLOR: Color = Color::Rgb(250, 177, 237);

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
/// Maximum log retention days.
pub const MAX_LOG_RETENTION_DAYS: Lazy<u64> = Lazy::new(|| {
    env::var("MAX_LOG_RETENTION_DAYS")
        .unwrap_or("7".to_string())
        .parse()
        .unwrap_or(7)
});

/// Guard for the non-blocking appender.
pub static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

/// Default string value.
pub const DEFAULT_STRING: &String = &String::new();

/*
AI stuff
 */

/*
Image
 */
/// Base URL for the AI image API.
pub const IMAGE_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}images/generations",
        env::var("AI_IMAGE_API_BASE_URL").unwrap_or_else(
            |_| env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});

/// Token for the AI image API.
pub const IMAGE_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_IMAGE_API_TOKEN").unwrap_or_else(|_| env::var("AI_API_TOKEN").unwrap_or_default())
});

/// Models for the AI image API.
pub const IMAGE_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_IMAGE_GENERATION_MODELS").unwrap_or_else(|_| "dall-e-3".to_string()));

/*
Chat and translation
 */
/// Base URL for the AI chat API.
pub const CHAT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}chat/completions",
        env::var("AI_CHAT_API_BASE_URL").unwrap_or_else(
            |_| env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});

/// Token for the AI chat API.
pub const CHAT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_CHAT_API_TOKEN").unwrap_or_else(|_| env::var("AI_API_TOKEN").unwrap_or_default())
});

/// Models for the AI chat API.
pub const CHAT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_CHAT_MODEL").unwrap_or("gpt-3.5-turbo".to_string()));

/*
Transcription
 */
/// Base URL for the AI transcription API.
pub const TRANSCRIPT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}audio/",
        env::var("AI_TRANSCRIPT_BASE_URL")
            .or_else(|_| env::var("AI_API_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1/".to_string())
    )
});

/// Token for the AI transcription API.
pub const TRANSCRIPT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_TRANSCRIPT_API_TOKEN")
        .or_else(|_| env::var("AI_API_TOKEN"))
        .unwrap_or_default()
});

/// Models for the AI transcription API.
pub const TRANSCRIPT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_TRANSCRIPT_MODELS").unwrap_or_else(|_| String::from("whisper-1")));

/*
gRPC server
*/
/// Flag to enable or disable the gRPC server.
pub const GRPC_IS_ON: Lazy<bool> =
    Lazy::new(|| env::var("GRPC_IS_ON").unwrap_or_else(|_| "false".to_string()) == "true");

/// Port for the gRPC server.
pub const GRPC_SERVER_PORT: Lazy<String> =
    Lazy::new(|| env::var("GRPC_SERVER_PORT").unwrap_or_else(|_| "8080  ".to_string()));
/// If the gRPC server should use TLS.
pub const GRPC_USE_TLS: Lazy<bool> =
    Lazy::new(|| env::var("USE_SSL").unwrap_or_else(|_| "false".to_string()) == "true");
/// Path to the gRPC server certificate.
pub const GRPC_CERT_PATH: Lazy<String> =
    Lazy::new(|| env::var("SSL_CERT_PATH").unwrap_or_else(|_| "cert/cert.pem".to_string()));
/// Path to the gRPC server key.
pub const GRPC_KEY_PATH: Lazy<String> =
    Lazy::new(|| env::var("SSL_KEY_PATH").unwrap_or_else(|_| "cert/key.pem".to_string()));

/// The version of the application, fetched from the environment variable "CARGO_PKG_VERSION".
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/*
Application
 */

/// If the application tui is enabled.
pub const APP_TUI: Lazy<bool> = Lazy::new(|| {
    let is_on = env::var("TUI").unwrap_or_else(|_| "false".to_string());
    is_on.to_lowercase() == "true"
});

/// User blacklist for the server image.
pub static mut USER_BLACKLIST_SERVER_IMAGE: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| {
    let user_ids: Vec<String> = Vec::new();
    Arc::from(RwLock::from(user_ids))
});
/// Db type.
pub const DB_TYPE: Lazy<String> =
    Lazy::new(|| env::var("DB_TYPE").unwrap_or_else(|_| "sqlite".to_string()));
/// Cache type.
pub const CACHE_TYPE: Lazy<String> =
    Lazy::new(|| env::var("CACHE_TYPE").unwrap_or_else(|_| "sqlite".to_string()));

/*
bot info
 */

/// The bot's information.
pub static mut BOT_INFO: Lazy<Option<CurrentApplicationInfo>> = Lazy::new(|| None);
/// Vec of all available bot commands.
pub const BOT_COMMANDS: Lazy<Vec<CommandItem>> = Lazy::new(get_list_of_all_command);
/// Used library.
pub const LIBRARY: Lazy<String> = Lazy::new(|| String::from("serenity"));

/*
federation stuff
 */

/// Is the federation server enabled.
pub const FEDERATION_IS_ON: Lazy<bool> =
    Lazy::new(|| env::var("FEDERATION_IS_ON").unwrap_or_else(|_| "false".to_string()) == "true");

/// The name of the federation server.
pub const NAME: Lazy<String> =
    Lazy::new(|| env::var("NAME").unwrap_or_else(|_| "kasuki".to_string()));

/// The federation server's address.
pub const FEDERATION_SERVER: Lazy<String> = Lazy::new(|| {
    env::var("FEDERATION_SERVER").unwrap_or_else(|_| "http://localhost:443".to_string())
});

/// The bot internet accessible address or url.
pub const SELF_URL: Lazy<String> =
    Lazy::new(|| env::var("SELF_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()));

/// Is the primary node of the federation.
pub const FEDERATION_IS_PRIMARY: Lazy<bool> = Lazy::new(|| {
    env::var("FEDERATION_IS_PRIMARY").unwrap_or_else(|_| "false".to_string()) == "true"
});

/// url of the primary node.
pub const FEDERATION_PRIMARY_SERVER: Lazy<String> = Lazy::new(|| {
    env::var("FEDERATION_PRIMARY_SERVER").unwrap_or_else(|_| "http://localhost:8080".to_string())
});
