use once_cell::sync::Lazy;
use serenity::all::Colour;

use crate::error_enum::AppError;
use crate::game_struct::steam_game_id_struct::App;

pub const DAYS: i64 = 3;
pub const ACTIVITY_NAME: &str = "Let you get info from anilist.";
pub const COLOR: Colour = Colour::FABLED_PINK;
pub const DATA_SQLITE_DB: &str = "./data.db";
pub const CACHE_SQLITE_DB: &str = "./cache.db";
pub const PING_UPDATE_DELAYS: u64 = 600;

pub static OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::OptionError(String::from("The option contain no value")));
pub static COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::CommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
pub static DIFFERED_COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::DifferedCommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
pub static DIFFERED_OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::DifferedOptionError(String::from("The option contain no value")));
pub static AUTOCOMPLETE_COUNT: u32 = 20;
pub static OTHER_CRATE_LEVEL: &str = "warn";
pub static UNKNOWN: &str = "Unknown";
pub static VERSION: &str = "V2.1.4";
pub static mut APPS: Vec<App> = Vec::new();
pub static GAME_UPDATE: u32 = 1;
