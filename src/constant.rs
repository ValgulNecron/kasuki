use once_cell::sync::Lazy;
use serenity::all::Colour;
use std::collections::HashMap;

use crate::error_enum::AppError;

pub const DAYS: i64 = 3;
pub const ACTIVITY_NAME: &str = "Let you get info from anilist.";
pub const COLOR: Colour = Colour::FABLED_PINK;
pub const DATA_SQLITE_DB: &str = "./data.db";
pub const CACHE_SQLITE_DB: &str = "./cache.db";
pub const PING_UPDATE_DELAYS: u64 = 600;

pub const OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::OptionError(String::from("The option contain no value")));
pub const COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::CommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
pub const DIFFERED_COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::DifferedCommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
pub const DIFFERED_OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::DifferedOptionError(String::from("The option contain no value")));
pub const AUTOCOMPLETE_COUNT: u32 = 20;
pub const OTHER_CRATE_LEVEL: &str = "warn";
pub const UNKNOWN: &str = "Unknown";
pub const VERSION: &str = "V2.2.0";
pub static mut APPS: Lazy<HashMap<String, u128>> = Lazy::new(HashMap::new);
pub const GAME_UPDATE: u32 = 1;
pub const PASS_LIMIT: u32 = 10;
pub const MEMBER_LIST_LIMIT: u64 = 11;
// min 2 max 1001 (yeah you should do -1 dunno why but yeah

pub const ACTIVITY_LIST_LIMIT: u64 = 10;
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
// in minute
pub const USER_COLOR_UPDATE_TIME: u32 = 30;
