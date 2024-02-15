use once_cell::sync::Lazy;
use serenity::all::Colour;
use std::collections::HashMap;
use std::env;

pub static ACTIVITY_NAME: Lazy<String> =
    Lazy::new(|| env::var("BOT_ACTIVITY").unwrap_or("Let you get info from anilist.".to_string()));
pub const APP_VERSION: &str = "V2.2.2";

/*
all delays
 */
pub const DELAY_BEFORE_THREAD_SPAWN: u64 = 30; // 30 seconds
pub const PING_UPDATE_DELAYS: u64 = 600; // 10 minutes
pub const TIME_BETWEEN_SERVER_IMAGE_UPDATE: u64 = 86_400; // 1 day
pub const TIME_BETWEEN_USER_COLOR_UPDATE: u64 = 1_800; // 30 minutes
pub const TIME_BETWEEN_GAME_UPDATE: u64 = 86_400; // 1 day
pub const TIME_BETWEEN_CACHE_UPDATE: u64 = 259_200; // 3 days

/*
limit.
 */
pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 20;
pub const PASS_LIMIT: u32 = 10;
// min 1 max 1000
pub const MEMBER_LIST_LIMIT: u64 = 10;
pub const ACTIVITY_LIST_LIMIT: u64 = 10;
/*
Database path.
 */
pub const DATA_SQLITE_DB: &str = "./data.db";
pub const CACHE_SQLITE_DB: &str = "./cache.db";
/*
app embed color.
 */
pub const COLOR: Colour = Colour::FABLED_PINK;
/*
other crate log level. because serenity use info for obviously trace thing like heartbeat.
 */
pub const OTHER_CRATE_LEVEL: &str = "warn";
/*
default value.
 */
pub const UNKNOWN: &str = "Unknown";
pub static LANG_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    [
        ("en", "english"),
        ("fr", "french"),
        ("de", "german"),
        ("ja", "japanese"),
    ]
    .iter()
    .cloned()
    .collect()
});

pub static mut APPS: Lazy<HashMap<String, u128>> = Lazy::new(HashMap::new);

pub const LOGS_PATH: &str = "./logs";
pub const LOGS_PREFIX: &str = "kasuki.log";
pub static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
pub static mut MAX_LOG_RETENTION_DAYS: u64 = 7;
pub const SERVER_IMAGE_PATH: &str = "server_image";
pub const DEFAULT_STRING: &String = &String::new();

/*
AI stuff
 */

/*
image
 */

pub static mut IMAGE_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}images/generations",
        env::var("AI_IMAGE_API_BASE_URL").unwrap_or(
            env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});
pub static mut IMAGE_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_IMAGE_API_TOKEN").unwrap_or(env::var("AI_API_TOKEN").unwrap_or_default())
});

pub static mut IMAGE_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_IMAGE_GENERATION_MODELS").unwrap_or(String::from("dall-e-3")));

/*
chat and translation
 */

pub static mut CHAT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}chat/completions",
        env::var("AI_CHAT_API_BASE_URL").unwrap_or(
            env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});
pub static mut CHAT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_CHAT_API_TOKEN").unwrap_or(env::var("AI_API_TOKEN").unwrap_or_default())
});

pub static mut CHAT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_CHAT_MODEL").unwrap_or(String::from("gpt-3.5-turbo")));

/*
transcription
 */

pub static mut TRANSCRIPT_BASE_URL: Lazy<String> = Lazy::new(|| {
    format!(
        "{}images/generations",
        env::var("AI_TRANSCRIPT_BASE_URL").unwrap_or(
            env::var("AI_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string())
        )
    )
});
pub static mut TRANSCRIPT_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("AI_TRANSCRIPT_API_TOKEN").unwrap_or(env::var("AI_API_TOKEN").unwrap_or_default())
});

pub static mut TRANSCRIPT_MODELS: Lazy<String> =
    Lazy::new(|| env::var("AI_TRANSCRIPT_MODELS").unwrap_or(String::from("whisper-1")));
