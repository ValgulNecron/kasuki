use std::collections::HashSet;

use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

use crate::anilist_struct::run::user::{
    Anime, Genre, Manga, Statistics, Statuses, Tag, UserWrapper,
};
use crate::command_run::anilist::user::get_user_data;
use crate::common::get_option::subcommand::get_option_map_string_subcommand;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::compare::load_localization_compare;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("username"))
        .cloned()
        .unwrap_or(String::new());
    let value2 = map
        .get(&String::from("username2"))
        .cloned()
        .unwrap_or(String::new());

    let data: UserWrapper = get_user_data(&value).await?;

    let data2: UserWrapper = get_user_data(&value2).await?;

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let compare_localised = load_localization_compare(guild_id).await?;

    let user = data.data.user.clone();
    let user2 = data2.data.user.clone();
    let username = user.name.clone().unwrap_or_default();
    let username2 = user2.name.clone().unwrap_or_default();

    let mut desc = String::new();

    let affinity = get_affinity(user.statistics.clone(), user2.statistics.clone());

    desc.push_str(
        compare_localised
            .affinity
            .replace("$1$", username.as_str())
            .replace("$2$", username2.as_str())
            .replace("$3$", affinity.to_string().as_str())
            .as_str(),
    );

    match user
        .statistics
        .anime
        .count
        .unwrap_or(0)
        .cmp(&user2.statistics.anime.count.unwrap_or(0))
    {
        std::cmp::Ordering::Greater => desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", username.as_str())
                .replace("$lesser$", username2.as_str())
                .as_str(),
        ),
        std::cmp::Ordering::Less => desc.push_str(
            compare_localised
                .more_anime
                .replace("$greater$", username2.as_str())
                .replace("$lesser$", username.as_str())
                .as_str(),
        ),
        std::cmp::Ordering::Equal => desc.push_str(
            compare_localised
                .same_anime
                .replace("$2$", username2.as_str())
                .replace("$1$", username.as_str())
                .as_str(),
        ),
    }

    match user
        .statistics
        .anime
        .minutes_watched
        .unwrap_or(0)
        .cmp(&user2.statistics.anime.minutes_watched.unwrap_or(0))
    {
        std::cmp::Ordering::Greater => desc.push_str(
            compare_localised
                .more_watch_time
                .replace("$greater$", username.as_str())
                .replace("$lesser$", username2.as_str())
                .as_str(),
        ),
        std::cmp::Ordering::Less => desc.push_str(
            compare_localised
                .more_watch_time
                .replace("$greater$", username2.as_str())
                .replace("$lesser$", username.as_str())
                .as_str(),
        ),
        std::cmp::Ordering::Equal => desc.push_str(
            compare_localised
                .same_watch_time
                .replace("$2$", username2.as_str())
                .replace("$1$", username.as_str())
                .as_str(),
        ),
    }

    let tag = get_tag(&user.statistics.anime.tags);
    let tag2 = get_tag(&user2.statistics.anime.tags);

    desc.push_str(
        diff(
            &tag,
            &tag2,
            &compare_localised.tag_anime,
            &compare_localised.same_tag_anime,
            &username,
            &username2,
        )
            .as_str(),
    );

    let genre = get_genre(&user.statistics.anime.genres);
    let genre2 = get_genre(&user2.statistics.anime.genres);

    desc.push_str(
        diff(
            &genre,
            &genre2,
            &compare_localised.genre_anime,
            &compare_localised.same_genre_anime,
            &username,
            &username2,
        )
            .as_str(),
    );

    match user
        .statistics
        .manga
        .count
        .unwrap_or(0)
        .cmp(&user2.statistics.manga.count.unwrap_or(0))
    {
        std::cmp::Ordering::Greater => {
            desc.push_str(
                compare_localised
                    .more_manga
                    .replace("$greater$", username.as_str())
                    .replace("$lesser$", username2.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Less => {
            desc.push_str(
                compare_localised
                    .more_manga
                    .replace("$greater$", username2.as_str())
                    .replace("$lesser$", username.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Equal => {
            desc.push_str(
                compare_localised
                    .same_manga
                    .replace("$2$", username2.as_str())
                    .replace("$1$", username.as_str())
                    .as_str(),
            );
        }
    }

    match user
        .statistics
        .manga
        .chapters_read
        .unwrap_or(0)
        .cmp(&user2.statistics.manga.chapters_read.unwrap_or(0))
    {
        std::cmp::Ordering::Greater => {
            desc.push_str(
                compare_localised
                    .more_manga_chapter
                    .replace("$greater$", username.as_str())
                    .replace("$lesser$", username2.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Less => {
            desc.push_str(
                compare_localised
                    .more_manga_chapter
                    .replace("$greater$", username2.as_str())
                    .replace("$lesser$", username.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Equal => {
            desc.push_str(
                compare_localised
                    .same_manga_chapter
                    .replace("$2$", username2.as_str())
                    .replace("$1$", username.as_str())
                    .as_str(),
            );
        }
    }

    let tag = get_tag(&user.statistics.manga.tags);
    let tag2 = get_tag(&user2.statistics.manga.tags);

    desc.push_str(
        diff(
            &tag,
            &tag2,
            &compare_localised.tag_manga,
            &compare_localised.same_genre_manga,
            &username,
            &username2,
        )
            .as_str(),
    );

    let genre = get_genre(&user.statistics.manga.genres);
    let genre2 = get_genre(&user2.statistics.manga.genres);

    desc.push_str(
        diff(
            &genre,
            &genre2,
            &compare_localised.genre_manga,
            &compare_localised.same_genre_manga,
            &username,
            &username2,
        )
            .as_str(),
    );

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

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

fn get_affinity(s1: Statistics, s2: Statistics) -> f64 {
    let mut affinity: f64;

    affinity = jaccard_index(&tag_string(&s1.anime.tags), &tag_string(&s2.anime.tags));

    trace!(affinity);

    affinity += jaccard_index(
        &genre_string(&s1.anime.genres),
        &genre_string(&s2.anime.genres),
    );

    trace!(affinity);

    let mut affinity2 = other_affinity_anime(s1.anime, s2.anime);

    trace!(affinity);

    affinity2 += other_affinity_manga(s1.manga, s2.manga);

    trace!(affinity);

    ((affinity / 2.0) + affinity2) * 100.0
}

fn other_affinity_anime(anime: Anime, anime0: Anime) -> f64 {
    let (current, planning, completed, dropped, paused, repeating) =
        get_number_by_status(anime.statuses);
    let (current0, planning0, completed0, dropped0, paused0, repeating0) =
        get_number_by_status(anime0.statuses);

    let mut affinity = 0.0;

    if current == current0 {
        affinity += 1f64
    }
    if planning == planning0 {
        affinity += 1f64
    }
    if completed == completed0 {
        affinity += 1f64
    }
    if dropped == dropped0 {
        affinity += 1f64
    }
    if paused == paused0 {
        affinity += 1f64
    }
    if repeating == repeating0 {
        affinity += 1f64
    }

    if anime.count.unwrap_or(0) == anime0.count.unwrap_or(0) {
        affinity += 1f64
    }

    if anime.minutes_watched.unwrap_or(0) == anime0.minutes_watched.unwrap_or(0) {
        affinity += 1f64
    }

    if anime.standard_deviation.unwrap_or(0.0) == anime0.standard_deviation.unwrap_or(0.0) {
        affinity += 1f64
    }

    if anime.mean_score.unwrap_or(0.0) == anime0.mean_score.unwrap_or(0.0) {
        affinity += 1f64
    }

    affinity / 20.0
}

fn other_affinity_manga(manga: Manga, manga0: Manga) -> f64 {
    let (current, planning, completed, dropped, paused, repeating) =
        get_number_by_status(manga.statuses);
    let (current0, planning0, completed0, dropped0, paused0, repeating0) =
        get_number_by_status(manga0.statuses);

    let mut affinity = 0.0;

    if current == current0 {
        affinity += 1f64
    }
    if planning == planning0 {
        affinity += 1f64
    }
    if completed == completed0 {
        affinity += 1f64
    }
    if dropped == dropped0 {
        affinity += 1f64
    }
    if paused == paused0 {
        affinity += 1f64
    }
    if repeating == repeating0 {
        affinity += 1f64
    }

    if manga.count.unwrap_or(0) == manga0.count.unwrap_or(0) {
        affinity += 1f64
    }

    if manga.chapters_read.unwrap_or(0) == manga0.chapters_read.unwrap_or(0) {
        affinity += 1f64
    }

    if manga.standard_deviation.unwrap_or(0.0) == manga0.standard_deviation.unwrap_or(0.0) {
        affinity += 1f64
    }

    if manga.mean_score.unwrap_or(0.0) == manga0.mean_score.unwrap_or(0.0) {
        affinity += 1f64
    }

    affinity / 20.0
}

fn jaccard_index(a: &[String], b: &[String]) -> f64 {
    let set_a: HashSet<_> = a.iter().collect();
    let set_b: HashSet<_> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    intersection as f64 / union as f64
}

fn get_number_by_status(s: Vec<Statuses>) -> (i32, i32, i32, i32, i32, i32) {
    let mut current = 0;
    let mut planning = 0;
    let mut completed = 0;
    let mut dropped = 0;
    let mut paused = 0;
    let mut repeating = 0;
    for statuses in s {
        match statuses.status.as_str() {
            "CURRENT" => current = statuses.count,
            "PLANNING" => planning = statuses.count,
            "COMPLETED" => completed = statuses.count,
            "DROPPED" => dropped = statuses.count,
            "PAUSED" => paused = statuses.count,
            "REPEATING" => repeating = statuses.count,
            _ => {}
        }
    }
    (current, planning, completed, dropped, paused, repeating)
}

fn tag_string(vec: &[Tag]) -> Vec<String> {
    vec.iter()
        .map(|tag| tag.tag.name.clone().unwrap())
        .collect()
}

fn genre_string(vec: &[Genre]) -> Vec<String> {
    vec.iter()
        .map(|genre| genre.genre.clone().unwrap())
        .collect()
}

fn get_tag(tags: &[Tag]) -> String {
    if tags.len() > 1 {
        tags[0].tag.name.clone().unwrap_or_default()
    } else {
        String::new()
    }
}

fn get_genre(genres: &[Genre]) -> String {
    if genres.len() > 1 {
        genres[0].genre.clone().unwrap_or_default()
    } else {
        String::new()
    }
}

fn diff(
    a1: &str,
    a2: &str,
    diff_text: &str,
    same: &str,
    username: &str,
    username2: &str,
) -> String {
    let diff = a1 != a2;

    let info = if diff {
        diff_text
            .replace("$1$", username)
            .replace("$2$", username2)
            .replace("$1a$", a1)
            .replace("$2a$", a2)
    } else {
        same.replace("$1$", username)
            .replace("$2$", username2)
            .replace("$1a$", a1)
    };
    trace!(info);
    info
}
