use serenity::utils::Colour;

/// The number of days
///
/// This constant represents the number of days. It is of type `i64`.
pub const DAYS: i64 = 3;
///
/// This constant represents the name of the activity. It is of type `&str` (string slice).
/// The activity is the text display in the bot status.
pub const ACTIVITY_NAME: &str = "Let you get info from anilist.";
/// The color constant
///
/// This constant represents the color `Colour::FABLED_PINK`. It is of type `Colour`.
pub const COLOR: Colour = Colour::FABLED_PINK;
/// Not Applicable
///
/// This constant represents the special value "N/A". It is of type `&str`.
pub const N_A: &str = "N/A";
pub const DATA_SQLITE_DB: &str = "./data.db";
pub const CACHE_SQLITE_DB: &str = "./cache.db";
