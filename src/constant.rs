use serenity::utils::Colour;

/// The number of days
///
/// This constant represents the number of days before the cache is considered out of data.
/// It is of type `i64`.
pub const DAYS: i64 = 3;
/// This constant represents the name of the activity. It is of type `&str` (string slice).
/// The activity is the text display in the bot status.
pub const ACTIVITY_NAME: &str = "Let you get info from anilist.";
/// The color constant
///
/// This constant represents the color `Colour::FABLED_PINK`.
/// it is used for embed.
/// It is of type `Colour`.
pub const COLOR: Colour = Colour::FABLED_PINK;
/// Not Applicable
///
/// This constant represents the special value "N/A". It is of type `&str`.
pub const N_A: &str = "N/A";
/// The path to the SQLite database
///
/// This constant represents the path to the SQLite database file that will be used by the code.
/// It is of type `&str`.
pub const DATA_SQLITE_DB: &str = "./data.db";
/// The path to the cache SQLite database
///
/// This constant represents the file path to the SQLite database used for caching.
/// It is of type `&str`.
pub const CACHE_SQLITE_DB: &str = "./cache.db";
/// The ping update delay
///
/// This constant represents the delay in seconds before a ping update is triggered.
/// It is of type `u64`.
pub const PING_UPDATE_DELAYS: u64 = 600;
