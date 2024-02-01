use once_cell::sync::Lazy;
use serenity::all::Colour;
use std::collections::HashMap;
use std::env;

pub static ACTIVITY_NAME: Lazy<String> = Lazy::new(|| {
    let activity = env::var("BOT_ACTIVITY").unwrap_or("Let you get info from anilist.".to_string());
    activity
});
pub const APP_VERSION: &str = "V2.2.0";

/*
all delays in seconds.
 */
pub const DELAY_BEFORE_THREAD_SPAWN: u64 = 30;
pub const PING_UPDATE_DELAYS: u64 = 600;
/*
all delays in minutes.
 */
pub const TIME_BETWEEN_USER_COLOR_UPDATE: u32 = 30;
/*
all delays in days.
 */
pub const TIME_BETWEEN_GAME_UPDATE: u32 = 1;
pub const TIME_BETWEEN_CACHE_UPDATE: i64 = 3;
/*
limit.
 */
pub const AUTOCOMPLETE_COUNT_LIMIT: u32 = 20;
pub const PASS_LIMIT: u32 = 10;
// min 2 max 1001 (yeah you should do +1 dunno why but yeah so 2 = 1 and 1001 = 1000)
pub const MEMBER_LIST_LIMIT: u64 = 11;
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
