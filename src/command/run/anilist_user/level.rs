use once_cell::sync::Lazy;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::command::run::anilist_user::user::get_user_data;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::get_registered_user;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anilist_user::level::load_localization_level;
use crate::structure::run::anilist::user::{get_color, get_completed, get_user_url, UserWrapper};

/// Executes the command to display a user's level based on their anime and manga statistics.
///
/// This function retrieves the username from the command interaction. If a username is provided, it fetches the user data and sends an embed containing the user's level and progress.
/// If no username is provided, it retrieves the ID of the user who triggered the command, checks if they are registered, and if they are, fetches their user data and sends an embed containing their level and progress.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the username from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let user = map.get(&String::from("username"));
    match user {
        Some(value) => {
            // If a username is provided, fetch the user data and send an embed
            let data: UserWrapper = get_user_data(value).await?;
            send_embed(ctx, command_interaction, data).await
        }
        None => {
            // If no username is provided, retrieve the ID of the user who triggered the command
            let user_id = &command_interaction.user.id.to_string();
            // Check if the user is registered
            let row: Option<RegisteredUser> = get_registered_user(user_id).await?;
            let user = row.ok_or(AppError::new(
                String::from("There is no user selected"),
                ErrorType::Option,
                ErrorResponseType::Message,
            ))?;

            // Fetch the user data and send an embed
            let data: UserWrapper = get_user_data(&user.anilist_id).await?;
            send_embed(ctx, command_interaction, data).await
        }
    }
}

/// Sends an embed containing a user's level and progress based on their anime and manga statistics.
///
/// This function calculates the user's level and progress based on their anime and manga statistics, constructs an embed containing this information, and sends it as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `data` - The user data to use for calculating the level and progress.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: UserWrapper,
) -> Result<(), AppError> {
    // Get the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized level strings
    let level_localised = load_localization_level(guild_id).await?;

    // Clone the user data
    let user = data.data.user.clone();

    // Clone the manga and anime statistics
    let manga = user.statistics.manga.clone();
    let anime = user.statistics.anime.clone();

    // Calculate the number of manga and anime completed
    let manga_completed = get_completed(manga.statuses.clone());
    let anime_completed = get_completed(anime.statuses.clone());
    // Get the number of chapters read and minutes watched
    let chap_read = manga.chapters_read.unwrap_or(0);
    let tw = anime.minutes_watched.unwrap_or(0);

    // Calculate the experience points
    let xp =
        (2.0 * (manga_completed + anime_completed) as f64) + chap_read as f64 + (tw as f64 * 0.1);

    // Get the username
    let username = user.name.clone().unwrap();

    // Calculate the level and progress
    let (level, actual, next_xp): (u32, f64, f64) = get_level(xp);

    // Initialize the embed builder
    let mut builder_embed = get_default_embed(Some(get_color(user.clone())))
        .title(user.name.unwrap_or_default())
        .url(get_user_url(user.id.unwrap_or(0)))
        .thumbnail(user.avatar.large.clone().unwrap())
        .description(
            level_localised
                .desc
                .replace("$username$", username.as_str())
                .replace("$level$", level.to_string().as_str())
                .replace("$xp$", xp.to_string().as_str())
                .replace("$actual$", actual.to_string().as_str())
                .replace("$next$", next_xp.to_string().as_str()),
        );

    // Add the user's banner image to the embed if it exists
    match &user.banner_image {
        Some(banner_image) => builder_embed = builder_embed.image(banner_image),
        None => {}
    }

    // Create the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Create the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}

pub static LEVELS: Lazy<[(u32, f64, f64); 102]> = Lazy::new(|| {
    [
        (0, 0.0, xp_required_for_level(1)),
        (1, xp_required_for_level(1), xp_required_for_level(2)),
        (2, xp_required_for_level(2), xp_required_for_level(3)),
        (3, xp_required_for_level(3), xp_required_for_level(4)),
        (4, xp_required_for_level(4), xp_required_for_level(5)),
        (5, xp_required_for_level(5), xp_required_for_level(6)),
        (6, xp_required_for_level(6), xp_required_for_level(7)),
        (7, xp_required_for_level(7), xp_required_for_level(8)),
        (8, xp_required_for_level(8), xp_required_for_level(9)),
        (9, xp_required_for_level(9), xp_required_for_level(10)),
        (10, xp_required_for_level(10), xp_required_for_level(11)),
        (11, xp_required_for_level(11), xp_required_for_level(12)),
        (12, xp_required_for_level(12), xp_required_for_level(13)),
        (13, xp_required_for_level(13), xp_required_for_level(14)),
        (14, xp_required_for_level(14), xp_required_for_level(15)),
        (15, xp_required_for_level(15), xp_required_for_level(16)),
        (16, xp_required_for_level(16), xp_required_for_level(17)),
        (17, xp_required_for_level(17), xp_required_for_level(18)),
        (18, xp_required_for_level(18), xp_required_for_level(19)),
        (19, xp_required_for_level(19), xp_required_for_level(20)),
        (20, xp_required_for_level(20), xp_required_for_level(21)),
        (21, xp_required_for_level(21), xp_required_for_level(22)),
        (22, xp_required_for_level(22), xp_required_for_level(23)),
        (23, xp_required_for_level(23), xp_required_for_level(24)),
        (24, xp_required_for_level(24), xp_required_for_level(25)),
        (25, xp_required_for_level(25), xp_required_for_level(26)),
        (26, xp_required_for_level(26), xp_required_for_level(27)),
        (27, xp_required_for_level(27), xp_required_for_level(28)),
        (28, xp_required_for_level(28), xp_required_for_level(29)),
        (29, xp_required_for_level(29), xp_required_for_level(30)),
        (30, xp_required_for_level(30), xp_required_for_level(31)),
        (31, xp_required_for_level(31), xp_required_for_level(32)),
        (32, xp_required_for_level(32), xp_required_for_level(33)),
        (33, xp_required_for_level(33), xp_required_for_level(34)),
        (34, xp_required_for_level(34), xp_required_for_level(35)),
        (35, xp_required_for_level(35), xp_required_for_level(36)),
        (36, xp_required_for_level(36), xp_required_for_level(37)),
        (37, xp_required_for_level(37), xp_required_for_level(38)),
        (38, xp_required_for_level(38), xp_required_for_level(39)),
        (39, xp_required_for_level(39), xp_required_for_level(40)),
        (40, xp_required_for_level(40), xp_required_for_level(41)),
        (41, xp_required_for_level(41), xp_required_for_level(42)),
        (42, xp_required_for_level(42), xp_required_for_level(43)),
        (43, xp_required_for_level(43), xp_required_for_level(44)),
        (44, xp_required_for_level(44), xp_required_for_level(45)),
        (45, xp_required_for_level(45), xp_required_for_level(46)),
        (46, xp_required_for_level(46), xp_required_for_level(47)),
        (47, xp_required_for_level(47), xp_required_for_level(48)),
        (48, xp_required_for_level(48), xp_required_for_level(49)),
        (49, xp_required_for_level(49), xp_required_for_level(50)),
        (50, xp_required_for_level(50), xp_required_for_level(51)),
        (51, xp_required_for_level(51), xp_required_for_level(52)),
        (52, xp_required_for_level(52), xp_required_for_level(53)),
        (53, xp_required_for_level(53), xp_required_for_level(54)),
        (54, xp_required_for_level(54), xp_required_for_level(55)),
        (55, xp_required_for_level(55), xp_required_for_level(56)),
        (56, xp_required_for_level(56), xp_required_for_level(57)),
        (57, xp_required_for_level(57), xp_required_for_level(58)),
        (58, xp_required_for_level(58), xp_required_for_level(59)),
        (59, xp_required_for_level(59), xp_required_for_level(60)),
        (60, xp_required_for_level(60), xp_required_for_level(61)),
        (61, xp_required_for_level(61), xp_required_for_level(62)),
        (62, xp_required_for_level(62), xp_required_for_level(63)),
        (63, xp_required_for_level(63), xp_required_for_level(64)),
        (64, xp_required_for_level(64), xp_required_for_level(65)),
        (65, xp_required_for_level(65), xp_required_for_level(66)),
        (66, xp_required_for_level(66), xp_required_for_level(67)),
        (67, xp_required_for_level(67), xp_required_for_level(68)),
        (68, xp_required_for_level(68), xp_required_for_level(69)),
        (69, xp_required_for_level(69), xp_required_for_level(70)),
        (70, xp_required_for_level(70), xp_required_for_level(71)),
        (71, xp_required_for_level(71), xp_required_for_level(72)),
        (72, xp_required_for_level(72), xp_required_for_level(73)),
        (73, xp_required_for_level(73), xp_required_for_level(74)),
        (74, xp_required_for_level(74), xp_required_for_level(75)),
        (75, xp_required_for_level(75), xp_required_for_level(76)),
        (76, xp_required_for_level(76), xp_required_for_level(77)),
        (77, xp_required_for_level(77), xp_required_for_level(78)),
        (78, xp_required_for_level(78), xp_required_for_level(79)),
        (79, xp_required_for_level(79), xp_required_for_level(80)),
        (80, xp_required_for_level(80), xp_required_for_level(81)),
        (81, xp_required_for_level(81), xp_required_for_level(82)),
        (82, xp_required_for_level(82), xp_required_for_level(83)),
        (83, xp_required_for_level(83), xp_required_for_level(84)),
        (84, xp_required_for_level(84), xp_required_for_level(85)),
        (85, xp_required_for_level(85), xp_required_for_level(86)),
        (86, xp_required_for_level(86), xp_required_for_level(87)),
        (87, xp_required_for_level(87), xp_required_for_level(88)),
        (88, xp_required_for_level(88), xp_required_for_level(89)),
        (89, xp_required_for_level(89), xp_required_for_level(90)),
        (90, xp_required_for_level(90), xp_required_for_level(91)),
        (91, xp_required_for_level(91), xp_required_for_level(92)),
        (92, xp_required_for_level(92), xp_required_for_level(93)),
        (93, xp_required_for_level(93), xp_required_for_level(94)),
        (94, xp_required_for_level(94), xp_required_for_level(95)),
        (95, xp_required_for_level(95), xp_required_for_level(96)),
        (96, xp_required_for_level(96), xp_required_for_level(97)),
        (97, xp_required_for_level(97), xp_required_for_level(98)),
        (98, xp_required_for_level(98), xp_required_for_level(99)),
        (99, xp_required_for_level(99), xp_required_for_level(100)),
        (100, xp_required_for_level(100), f64::MAX),
        (101, f64::MAX, f64::MAX),
    ]
});

/// Calculates the level, progress within that level, and the total progress required to reach the next level based on the given experience points (xp).
///
/// This function iterates over the predefined LEVELS array in reverse order. For each level, it checks if the given xp is greater than or equal to the required xp for that level.
/// If it is, it calculates the progress within that level and the total progress required to reach the next level, and returns these values along with the level.
/// If the given xp is less than the required xp for all levels, it returns 0 for the level and progress, and 20.0 for the total progress required to reach the next level.
///
/// # Arguments
///
/// * `xp` - The experience points for which to calculate the level and progress.
///
/// # Returns
///
/// A tuple containing the level as a `u32`, the progress within that level as a `f64`, and the total progress required to reach the next level as a `f64`.
fn get_level(xp: f64) -> (u32, f64, f64) {
    for &(level, required_xp, next_level_required_xp) in LEVELS.iter().rev() {
        if xp >= required_xp {
            let level_progress = xp - required_xp;
            let level_progress_total = next_level_required_xp - required_xp;
            return (level, level_progress, level_progress_total);
        }
    }
    (0, 0.0, 20.0)
}

/// Calculates the experience points required to reach a given level.
///
/// This function uses a match expression to determine the formula to use based on the given level.
/// The formulas are as follows:
/// - For levels 0 to 9, the cube of the level is used.
/// - For levels 10 to 29, the fourth power of the level is used.
/// - For levels 30 to 39, the fifth power of the level is used.
/// - For levels 40 to 49, the sixth power of the level is used.
/// - For levels 50 to 59, the seventh power of the level is used.
/// - For levels 60 to 69, the eighth power of the level is used.
/// - For levels 70 to 79, the ninth power of the level is used.
/// - For levels 80 to 89, the tenth power of the level is used.
/// - For levels 90 to 100, the eleventh power of the level is used.
/// - For levels above 100, the maximum possible `f64` value is used.
///
/// # Arguments
///
/// * `level` - The level for which to calculate the required experience points.
///
/// # Returns
///
/// The experience points required to reach the given level as a `f64`.
fn xp_required_for_level(level: u32) -> f64 {
    match level {
        0..=9 => (level as f64).powf(3f64),
        10..=29 => (level as f64).powf(4f64),
        30..=39 => (level as f64).powf(5f64),
        40..=49 => (level as f64).powf(6f64),
        50..=59 => (level as f64).powf(7f64),
        60..=69 => (level as f64).powf(8f64),
        70..=79 => (level as f64).powf(9f64),
        80..=89 => (level as f64).powf(10f64),
        90..=100 => (level as f64).powf(11f64),
        _ => f64::MAX,
    }
}
