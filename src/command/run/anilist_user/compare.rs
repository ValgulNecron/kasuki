use std::collections::HashSet;

use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::trace;

use crate::structure::run::anilist::user::{
    Anime, Genre, Manga, Statistics, Statuses, Tag, UserWrapper,
};
use crate::command::run::anilist_user::user::get_user_data;
use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::structure::message::anilist_user::compare::load_localization_compare;

/// Executes the comparison between two users' anime and manga statistics.
///
/// This function retrieves the usernames from the command interaction, fetches the user data for both users,
/// and calculates the affinity between them. It then constructs a description string that includes the affinity
/// and comparisons of various statistics between the two users. This description is sent as a response to the command interaction.
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
    // Retrieve the usernames from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("username"))
        .cloned()
        .unwrap_or(String::new());
    let value2 = map
        .get(&String::from("username2"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the user data for both users
    let data: UserWrapper = get_user_data(&value).await?;
    let data2: UserWrapper = get_user_data(&value2).await?;

    // Get the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized comparison strings
    let compare_localised = load_localization_compare(guild_id).await?;

    // Clone the user data
    let user = data.data.user.clone();
    let user2 = data2.data.user.clone();
    let username = user.name.clone().unwrap_or_default();
    let username2 = user2.name.clone().unwrap_or_default();

    // Initialize the description string
    let mut desc = String::new();

    // Calculate the affinity between the two users
    let affinity = get_affinity(user.statistics.clone(), user2.statistics.clone());

    // Add the affinity to the description string
    desc.push_str(
        compare_localised
            .affinity
            .replace("$1$", username.as_str())
            .replace("$2$", username2.as_str())
            .replace("$3$", affinity.to_string().as_str())
            .as_str(),
    );

    // Compare the count of anime watched by the two users and add the result to the description string
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

    // Compare the minutes watched by the two users and add the result to the description string
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

    // Get the tags of the anime watched by the two users and add the comparison to the description string
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

    // Get the genres of the anime watched by the two users and add the comparison to the description string
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

    // Compare the count of manga read by the two users and add the result to the description string
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

    // Compare the chapters read by the two users and add the result to the description string
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

    // Get the tags of the manga read by the two users and add the comparison to the description string
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

    // Get the genres of the manga read by the two users and add the comparison to the description string
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

    // Create the embed for the response
    let builder_embed = get_default_embed(None).description(desc);

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

/// Calculates the affinity between two users based on their anime and manga statistics.
///
/// The affinity is calculated based on the following factors:
/// - The Jaccard index of the tags of the anime watched by the users.
/// - The Jaccard index of the genres of the anime watched by the users.
/// - The affinity between the anime watched by the users.
/// - The affinity between the manga read by the users.
///
/// The total affinity is the sum of the Jaccard indices and the anime and manga affinities, divided by 2, and multiplied by 100.
///
/// # Arguments
///
/// * `s1` - The statistics of the first user.
/// * `s2` - The statistics of the second user.
///
/// # Returns
///
/// A `f64` representing the affinity between the two users.
fn get_affinity(s1: Statistics, s2: Statistics) -> f64 {
    // Initialize the affinity
    let mut affinity: f64;

    // Calculate the Jaccard index of the tags of the anime watched by the users
    affinity = jaccard_index(&tag_string(&s1.anime.tags), &tag_string(&s2.anime.tags));

    // Log the current affinity
    trace!(affinity);

    // Add the Jaccard index of the genres of the anime watched by the users to the affinity
    affinity += jaccard_index(
        &genre_string(&s1.anime.genres),
        &genre_string(&s2.anime.genres),
    );

    // Log the current affinity
    trace!(affinity);

    // Calculate the affinity between the anime watched by the users
    let mut affinity2 = other_affinity_anime(s1.anime, s2.anime);

    // Log the current affinity
    trace!(affinity);

    // Add the affinity between the manga read by the users to the affinity
    affinity2 += other_affinity_manga(s1.manga, s2.manga);

    // Log the current affinity
    trace!(affinity);

    // Return the total affinity divided by 2 and multiplied by 100
    ((affinity / 2.0) + affinity2) * 100.0
}

/// Calculates the affinity between two anime based on their status and statistics.
///
/// The affinity is calculated based on the following factors:
/// - The status of the anime (current, planning, completed, dropped, paused, repeating)
/// - The count of the anime
/// - The minutes watched
/// - The standard deviation of the scores
/// - The mean score
///
/// Each factor contributes equally to the affinity. If two anime have the same value for a factor, the affinity is increased by 1.
/// The total affinity is the sum of all factors divided by 20.
///
/// # Arguments
///
/// * `anime` - The first anime to compare.
/// * `anime0` - The second anime to compare.
///
/// # Returns
///
/// A `f64` representing the affinity between the two anime.
fn other_affinity_anime(anime: Anime, anime0: Anime) -> f64 {
    // Retrieve the number of anime in each status category for both anime
    let (current, planning, completed, dropped, paused, repeating) =
        get_number_by_status(anime.statuses);
    let (current0, planning0, completed0, dropped0, paused0, repeating0) =
        get_number_by_status(anime0.statuses);

    // Initialize the affinity to 0
    let mut affinity = 0.0;

    // Increase the affinity by 1 for each matching status category
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

    // Increase the affinity by 1 if the count of the anime is the same
    if anime.count.unwrap_or(0) == anime0.count.unwrap_or(0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the minutes watched is the same
    if anime.minutes_watched.unwrap_or(0) == anime0.minutes_watched.unwrap_or(0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the standard deviation of the scores is the same
    if anime.standard_deviation.unwrap_or(0.0) == anime0.standard_deviation.unwrap_or(0.0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the mean score is the same
    if anime.mean_score.unwrap_or(0.0) == anime0.mean_score.unwrap_or(0.0) {
        affinity += 1f64
    }

    // Return the total affinity divided by 20
    affinity / 20.0
}

/// Calculates the affinity between two manga based on their status and statistics.
///
/// The affinity is calculated based on the following factors:
/// - The status of the manga (current, planning, completed, dropped, paused, repeating)
/// - The count of the manga
/// - The number of chapters read
/// - The standard deviation of the scores
/// - The mean score
///
/// Each factor contributes equally to the affinity. If two manga have the same value for a factor, the affinity is increased by 1.
/// The total affinity is the sum of all factors divided by 20.
///
/// # Arguments
///
/// * `manga` - The first manga to compare.
/// * `manga0` - The second manga to compare.
///
/// # Returns
///
/// A `f64` representing the affinity between the two manga.
fn other_affinity_manga(manga: Manga, manga0: Manga) -> f64 {
    // Retrieve the number of manga in each status category for both manga
    let (current, planning, completed, dropped, paused, repeating) =
        get_number_by_status(manga.statuses);
    let (current0, planning0, completed0, dropped0, paused0, repeating0) =
        get_number_by_status(manga0.statuses);

    // Initialize the affinity to 0
    let mut affinity = 0.0;

    // Increase the affinity by 1 for each matching status category
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

    // Increase the affinity by 1 if the count of the manga is the same
    if manga.count.unwrap_or(0) == manga0.count.unwrap_or(0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the number of chapters read is the same
    if manga.chapters_read.unwrap_or(0) == manga0.chapters_read.unwrap_or(0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the standard deviation of the scores is the same
    if manga.standard_deviation.unwrap_or(0.0) == manga0.standard_deviation.unwrap_or(0.0) {
        affinity += 1f64
    }

    // Increase the affinity by 1 if the mean score is the same
    if manga.mean_score.unwrap_or(0.0) == manga0.mean_score.unwrap_or(0.0) {
        affinity += 1f64
    }

    // Return the total affinity divided by 20
    affinity / 20.0
}

/// Calculates the Jaccard index between two sets of strings.
///
/// The Jaccard index, also known as Intersection over Union, is a measure of how similar two sets are.
/// It is calculated as the size of the intersection divided by the size of the union of the two sets.
///
/// # Arguments
///
/// * `a` - The first set of strings.
/// * `b` - The second set of strings.
///
/// # Returns
///
/// A `f64` representing the Jaccard index between the two sets.
fn jaccard_index(a: &[String], b: &[String]) -> f64 {
    let set_a: HashSet<_> = a.iter().collect();
    let set_b: HashSet<_> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    intersection as f64 / union as f64
}

/// Retrieves the number of anime/manga in each status category for a user.
///
/// The status categories are: "CURRENT", "PLANNING", "COMPLETED", "DROPPED", "PAUSED", "REPEATING".
///
/// # Arguments
///
/// * `s` - A vector of `Statuses` objects, each representing the status of a particular anime/manga.
///
/// # Returns
///
/// A tuple of six `i32` values, each representing the count of anime/manga in the corresponding status category.
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

/// Converts a vector of `Tag` references into a vector of `String`.
///
/// This function iterates over each `Tag` in the vector, clones its name, and unwraps it.
/// The result is a vector of `String` where each string is the name of a `Tag`.
///
/// # Arguments
///
/// * `vec` - A slice of `Tag` references.
///
/// # Returns
///
/// A `Vec<String>` containing the names of the `Tag`s.
fn tag_string(vec: &[Tag]) -> Vec<String> {
    vec.iter()
        .map(|tag| tag.tag.name.clone().unwrap())
        .collect()
}

/// Converts a vector of `Genre` references into a vector of `String`.
///
/// This function iterates over each `Genre` in the vector, clones its genre, and unwraps it.
/// The result is a vector of `String` where each string is the genre of a `Genre`.
///
/// # Arguments
///
/// * `vec` - A slice of `Genre` references.
///
/// # Returns
///
/// A `Vec<String>` containing the genres of the `Genre`s.
fn genre_string(vec: &[Genre]) -> Vec<String> {
    vec.iter()
        .map(|genre| genre.genre.clone().unwrap())
        .collect()
}

/// Retrieves the name of the first `Tag` in a slice of `Tag`s.
///
/// If the slice has more than one `Tag`, it clones and unwraps the name of the first `Tag`.
/// If the slice has one or no `Tag`s, it returns a new, empty `String`.
///
/// # Arguments
///
/// * `tags` - A slice of `Tag` references.
///
/// # Returns
///
/// A `String` containing the name of the first `Tag` or an empty `String`.
fn get_tag(tags: &[Tag]) -> String {
    if tags.len() > 1 {
        tags[0].tag.name.clone().unwrap_or_default()
    } else {
        String::new()
    }
}

/// Retrieves the genre of the first `Genre` in a slice of `Genre`s.
///
/// If the slice has more than one `Genre`, it clones and unwraps the genre of the first `Genre`.
/// If the slice has one or no `Genre`s, it returns a new, empty `String`.
///
/// # Arguments
///
/// * `genres` - A slice of `Genre` references.
///
/// # Returns
///
/// A `String` containing the genre of the first `Genre` or an empty `String`.
fn get_genre(genres: &[Genre]) -> String {
    if genres.len() > 1 {
        genres[0].genre.clone().unwrap_or_default()
    } else {
        String::new()
    }
}

/// Compares two strings and returns a formatted string based on their equality.
///
/// This function checks if the two input strings are equal.
/// If they are not equal, it replaces placeholders in `diff_text` with the input strings and their corresponding usernames.
/// If they are equal, it replaces placeholders in `same` with the input strings and their corresponding usernames.
///
/// # Arguments
///
/// * `a1` - The first string to compare.
/// * `a2` - The second string to compare.
/// * `diff_text` - The text to return if the strings are not equal.
/// * `same` - The text to return if the strings are equal.
/// * `username` - The username corresponding to the first string.
/// * `username2` - The username corresponding to the second string.
///
/// # Returns
///
/// A `String` containing the formatted `diff_text` or `same` based on the equality of the input strings.
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
