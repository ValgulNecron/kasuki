use once_cell::sync::Lazy;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

use crate::anilist_struct::run::user::{get_color, get_completed, get_user_url, UserWrapper};
use crate::command_run::anilist::user::get_user_data;
use crate::database::dispatcher::data_dispatch::get_registered_user;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{CommandSendingError, OptionError};
use crate::lang_struct::anilist::level::load_localization_level;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    trace!("{:?}", options);
    for option in options {
        if option.name.as_str() != "type" {
            if let Some(a) = option.value.as_str() {
                let value = &a.to_string();

                let data: UserWrapper = get_user_data(value).await?;

                return send_embed(ctx, command_interaction, data).await;
            }
        }
    }
    let user_id = &command_interaction.user.id.to_string();
    let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
    trace!("{:?}", row);
    let (user, _): (Option<String>, Option<String>) = row;
    let user = user.ok_or(Error(OptionError(String::from("There is no option"))))?;
    let data: UserWrapper = if user.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(user.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(&user).await?
    };
    send_embed(ctx, command_interaction, data).await
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    data: UserWrapper,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let level_localised = load_localization_level(guild_id).await?;

    let user = data.data.user.clone();

    let manga = user.statistics.manga.clone();
    let anime = user.statistics.anime.clone();

    let manga_completed = get_completed(manga.statuses.clone());
    let anime_completed = get_completed(anime.statuses.clone());
    let chap_read = manga.chapters_read.unwrap_or(0);
    let tw = anime.minutes_watched.unwrap_or(0);

    let xp =
        (2.0 * (manga_completed + anime_completed) as f64) + chap_read as f64 + (tw as f64 * 0.1);

    let username = user.name.clone().unwrap();

    let (level, actual, next_xp): (u32, f64, f64) = get_level(xp);

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(get_color(user.clone()))
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

    match &user.banner_image {
        Some(banner_image) => builder_embed = builder_embed.image(banner_image),
        None => {}
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            Error(CommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
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
