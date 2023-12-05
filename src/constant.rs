use crate::error_enum::AppError;
use once_cell::sync::Lazy;
use serenity::all::Colour;

pub const DAYS: i64 = 3;
pub const ACTIVITY_NAME: &str = "Let you get info from anilist.";
pub const COLOR: Colour = Colour::FABLED_PINK;
pub const N_A: &str = "N/A";
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
pub static NO_AVATAR_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::NoAvatarError(String::from("Error while getting the user avatar.")));
pub static DIFFERED_COMMAND_SENDING_ERROR: Lazy<AppError> = Lazy::new(|| {
    AppError::DifferedCommandSendingError(String::from(
        "Error while sending the response of the command.",
    ))
});
pub static DIFFERED_OPTION_ERROR: Lazy<AppError> =
    Lazy::new(|| AppError::DifferedOptionError(String::from("The option contain no value")));
pub static AUTOCOMPLETE_COUNT: u32 = 8;
