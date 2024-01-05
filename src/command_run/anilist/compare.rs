use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};
use std::collections::HashSet;
use tracing::trace;

use crate::anilist_struct::run::user::{
    Anime, Genre, Manga, Statistics, Statuses, Tag, UserWrapper,
};
use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use crate::lang_struct::anilist::compare::load_localization_compare;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let option = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;

    let value = match option {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let option2 = &options.get(1).ok_or(OPTION_ERROR.clone())?.value;

    let value2 = match option2 {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let data: UserWrapper = if value.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value).await?
    };

    let data2: UserWrapper = if value2.parse::<i32>().is_ok() {
        UserWrapper::new_user_by_id(value2.parse().unwrap()).await?
    } else {
        UserWrapper::new_user_by_search(value2).await?
    };
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
        std::cmp::Ordering::Greater => {
            desc.push_str(
                compare_localised
                    .more_anime
                    .replace("$greater$", username.as_str())
                    .replace("$lesser$", username2.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Less => {
            desc.push_str(
                compare_localised
                    .more_anime
                    .replace("$greater$", username2.as_str())
                    .replace("$lesser$", username.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Equal => {
            desc.push_str(
                compare_localised
                    .same_anime
                    .replace("$2$", username2.as_str())
                    .replace("$1$", username.as_str())
                    .as_str(),
            );
        }
    }

    match user
        .statistics
        .anime
        .minutes_watched
        .unwrap_or(0)
        .cmp(&user2.statistics.anime.minutes_watched.unwrap_or(0))
    {
        std::cmp::Ordering::Greater => {
            desc.push_str(
                compare_localised
                    .more_watch_time
                    .replace("$greater$", username.as_str())
                    .replace("$lesser$", username2.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Less => {
            desc.push_str(
                compare_localised
                    .more_watch_time
                    .replace("$greater$", username2.as_str())
                    .replace("$lesser$", username.as_str())
                    .as_str(),
            );
        }
        std::cmp::Ordering::Equal => {
            desc.push_str(
                compare_localised
                    .same_watch_time
                    .replace("$2$", username2.as_str())
                    .replace("$1$", username.as_str())
                    .as_str(),
            );
        }
    }

    let tag = if user.statistics.anime.tags.len() > 1 {
        user.statistics.anime.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let tag2 = if user2.statistics.anime.tags.len() > 1 {
        user2.statistics.anime.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diff = tag != tag2;

    if diff {
        desc.push_str(
            compare_localised
                .tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", tag.as_str())
                .replace("$2a$", tag2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", tag.as_str())
                .as_str(),
        )
    }

    let genre = if user.statistics.anime.genres.len() > 1 {
        user.statistics.anime.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let genre2 = if user2.statistics.anime.genres.len() > 1 {
        user2.statistics.anime.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diff = genre != genre2;

    if diff {
        desc.push_str(
            compare_localised
                .genre_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .replace("$2a$", genre2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_anime
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .as_str(),
        )
    }

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

    let tag = if user.statistics.manga.tags.len() > 1 {
        user.statistics.manga.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let tag2 = if user2.statistics.manga.tags.len() > 1 {
        user2.statistics.manga.tags[0]
            .tag
            .name
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diff = tag != tag2;

    if diff {
        desc.push_str(
            compare_localised
                .tag_manga
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", tag.as_str())
                .replace("$2a$", tag2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_manga
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", tag.as_str())
                .as_str(),
        )
    }

    let genre = if user.statistics.manga.genres.len() > 1 {
        user.statistics.manga.genres[0]
            .genre
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let genre2 = if user2.statistics.manga.genres.len() > 1 {
        user2.statistics.manga.genres[0]
            .genre
            .clone()
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diff = genre != genre2;

    if diff {
        desc.push_str(
            compare_localised
                .genre_manga
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .replace("$2a$", genre2.as_str())
                .as_str(),
        )
    } else {
        desc.push_str(
            compare_localised
                .same_tag_manga
                .replace("$1$", username.as_str())
                .replace("$2$", username2.as_str())
                .replace("$1a$", genre.as_str())
                .as_str(),
        )
    }

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(desc);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
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

fn jaccard_index(a: &Vec<String>, b: &Vec<String>) -> f64 {
    let set_a: HashSet<_> = a.into_iter().collect();
    let set_b: HashSet<_> = b.into_iter().collect();

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

fn tag_string(vec: &Vec<Tag>) -> Vec<String> {
    vec.into_iter()
        .map(|tag| tag.tag.name.clone().unwrap())
        .collect()
}

fn genre_string(vec: &Vec<Genre>) -> Vec<String> {
    vec.into_iter()
        .map(|genre| genre.genre.clone().unwrap())
        .collect()
}
