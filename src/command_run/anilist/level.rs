use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

use crate::anilist_struct::run::user::{get_color, get_completed, get_user_url, UserWrapper};
use crate::constant::{COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::lang_struct::anilist::level::load_localization_level;
use crate::sqls::general::data::get_registered_user;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    trace!("{:?}", options);
    for option in options {
        if option.name.as_str() != "type" {
            match option.value.as_str() {
                Some(a) => {
                    let value = &a.to_string();

                    let data: UserWrapper = if value.parse::<i32>().is_ok() {
                        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
                    } else {
                        UserWrapper::new_user_by_search(value).await?
                    };

                    return send_embed(ctx, command, data).await;
                }

                None => {}
            }
        }
    }
    let user_id = &command.user.id.to_string();
    let row: (Option<String>, Option<String>) = get_registered_user(user_id).await?;
    trace!("{:?}", row);
    let (user, _): (Option<String>, Option<String>) = row;
    let user = user.ok_or(OPTION_ERROR.clone())?;
    let data = UserWrapper::new_user_by_id((&user).parse::<i32>().unwrap()).await?;
    return send_embed(ctx, command, data).await;
}

pub async fn send_embed(
    ctx: &Context,
    command: &CommandInteraction,
    data: UserWrapper,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let level_localised = load_localization_level(guild_id).await?;

    let user = data.data.user.clone();

    let manga = user.statistics.manga.clone();
    let anime = user.statistics.anime.clone();

    let manga_completed = get_completed(manga.statuses.clone());
    let anime_completed = get_completed(anime.statuses.clone());
    let chap_read = manga.chapters_read.clone().unwrap_or(0);
    let tw = anime.minutes_watched.clone().unwrap_or(0);

    let xp =
        (2.0 * (manga_completed + anime_completed) as f64) + chap_read as f64 + (tw as f64 * 0.1);

    let username = user.name.clone().unwrap();

    let (level, actual, next_xp): (u32, f64, f64) = get_level(xp);

    let mut builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(get_color(user.clone()))
        .title(&user.name.unwrap_or(String::new()))
        .url(get_user_url(&user.id.clone().unwrap_or(0)))
        .thumbnail(&user.avatar.large.clone().unwrap())
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

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub const LEVELS: [(u32, f64, f64); 51] = [
    (0, 0.0, 20.0),
    (1, 20.0, 40.0),
    (2, 40.0, 60.0),
    (3, 60.0, 80.0),
    (4, 80.0, 100.0),
    (5, 100.0, 130.0),
    (6, 130.0, 160.0),
    (7, 160.0, 190.0),
    (8, 190.0, 220.0),
    (9, 220.0, 250.0),
    (10, 250.0, 280.0),
    (11, 280.0, 310.0),
    (12, 310.0, 340.0),
    (13, 340.0, 370.0),
    (14, 370.0, 400.0),
    (15, 9360.0, 13860.0),
    (16, 13860.0, 20460.0),
    (17, 20460.0, 30160.0),
    (18, 30160.0, 44360.0),
    (19, 44360.0, 65160.0),
    (20, 65160.0, 95560.0),
    (21, 95560.0, 140160.0),
    (22, 140160.0, 206160.0),
    (23, 206160.0, 303160.0),
    (24, 303160.0, 447160.0),
    (25, 447160.0, 657160.0),
    (26, 657160.0, 969160.0),
    (27, 969160.0, 1426160.0),
    (28, 1426160.0, 2096160.0),
    (29, 2096160.0, 3076160.0),
    (30, 3076160.0, 4526160.0),
    (31, 4526160.0, 6626160.0),
    (32, 6626160.0, 9746160.0),
    (33, 9746160.0, 14316160.0),
    (34, 14316160.0, 21016160.0),
    (35, 21016160.0, 30816160.0),
    (36, 30816160.0, 45316160.0),
    (37, 45316160.0, 66316160.0),
    (38, 66316160.0, 97516160.0),
    (39, 97516160.0, 143516160.0),
    (40, 143516160.0, 210516160.0),
    (41, 210516160.0, 308516160.0),
    (42, 308516160.0, 453516160.0),
    (43, 453516160.0, 663516160.0),
    (44, 663516160.0, 975516160.0),
    (45, 975516160.0, 1437516160.0),
    (46, 1437516160.0, 2107516160.0),
    (47, 2107516160.0, 3087516160.0),
    (48, 3087516160.0, 4537516160.0),
    (49, 4537516160.0, 6637516160.0),
    (50, 6637516160.0, 9757516160.0),
];

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
