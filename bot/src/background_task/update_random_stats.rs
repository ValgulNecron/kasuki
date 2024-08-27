use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::info;

use crate::constant::{RANDOM_STATS_PATH, TIME_BETWEEN_RANDOM_STATS_UPDATE};
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::site_statistic_anime::{AnimeStat, AnimeStatVariables};
use crate::structure::run::anilist::site_statistic_manga::{MangaStat, MangaStatVariables};

/// Represents the random statistics of anime and manga.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RandomStat {
    /// The last page of anime statistics.
    pub anime_last_page: i32,
    /// The last page of manga statistics.
    pub manga_last_page: i32,
}

impl Default for RandomStat {
    /// Returns a default `RandomStat` with `anime_last_page` set to 1796 and `manga_last_page` set to 1796.
    fn default() -> Self {
        Self {
            anime_last_page: 1796,
            manga_last_page: 1796,
        }
    }
}

/// Launches a background task to update the random statistics at regular intervals.
///
/// # Arguments
///
/// * `anilist_cache` - A cache for storing Anilist API responses.
pub async fn update_random_stats_launcher(anilist_cache: Arc<RwLock<Cache<String, String>>>) {
    // Log the start of the random stats update task.
    info!("Starting random stats update");

    // Create an interval that ticks every `TIME_BETWEEN_ACTIVITY_CHECK` seconds.
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_RANDOM_STATS_UPDATE));

    // Run the update task indefinitely.
    loop {
        // Wait for the next tick of the interval.
        interval.tick().await;

        // Update the random statistics and ignore the result.
        let _ = update_random_stats(anilist_cache.clone()).await;
    }
}

/// Updates the random statistics by fetching the latest statistics from the Anilist API and saving them to a JSON file.
///
/// # Arguments
///
/// * `anilist_cache` - A cache for storing Anilist API responses.
///
/// # Returns
///
/// Returns the updated `RandomStat` on success, or an error on failure.
pub async fn update_random_stats(
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat, Box<dyn Error>> {
    // Try to load random stats from a JSON file.
    let mut random_stats: RandomStat = match std::fs::read_to_string(RANDOM_STATS_PATH) {
        Ok(stats) => serde_json::from_str(&stats)?,
        Err(_) => RandomStat::default(),
    };

    // Update the random statistics.
    random_stats = update_random(random_stats, anilist_cache).await?;

    // Write the updated random statistics to a JSON file.
    let random_stats_json = serde_json::to_string(&random_stats)?;
    std::fs::write(RANDOM_STATS_PATH, random_stats_json)?;

    // Return the updated random statistics.
    Ok(random_stats)
}

/// Updates the random statistics by repeatedly calling `update_page` until there are no more pages to update.
///
/// # Arguments
///
/// * `random_stats` - The current random statistics.
/// * `anilist_cache` - A cache for storing Anilist API responses.
///
/// # Returns
///
/// A `Result` containing the updated random statistics or an error.
async fn update_random(
    mut random_stats: RandomStat,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat, Box<dyn Error>> {
    // Keep updating pages until there are no more pages to update.
    let mut has_more_pages = true;
    while has_more_pages {
        has_more_pages = update_page(&mut random_stats, &anilist_cache, true, true).await;
        // sleep 1s
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    has_more_pages = true;
    while has_more_pages {
        has_more_pages = update_page(&mut random_stats, &anilist_cache, false, false).await;
        // sleep 1s
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(random_stats)
}

/// Updates a single page of random statistics.
///
/// # Arguments
///
/// * `random_stats` - The current random statistics.
/// * `anilist_cache` - A cache for storing Anilist API responses.
/// * `update_anime` - Whether to update the anime statistics.
/// * `update_manga` - Whether to update the manga statistics.
///
/// # Returns
///
/// A boolean indicating whether there are more pages to update.
async fn update_page(
    random_stats: &mut RandomStat,
    anilist_cache: &Arc<RwLock<Cache<String, String>>>,
    update_anime: bool,
    update_manga: bool,
) -> bool {
    // Build the appropriate query based on whether we're updating anime or manga.
    let data = if update_anime {
        let var = AnimeStatVariables {
            page: Some(random_stats.anime_last_page),
        };
        let operation = AnimeStat::build(var);
        let data: Result<GraphQlResponse<AnimeStat>, Box<dyn Error>> =
            make_request_anilist(operation, false, anilist_cache.clone()).await;
        data
    } else if update_manga {
        let var = MangaStatVariables {
            page: Some(random_stats.manga_last_page),
        };
        let operation = MangaStat::build(var);
        let data: Result<GraphQlResponse<AnimeStat>, Box<dyn Error>> =
            make_request_anilist(operation, false, anilist_cache.clone()).await;
        data
    } else {
        return false;
    };

    // Extract the data from the result. If there was an error, return false.
    let data = match data {
        Ok(data) => data,
        Err(_) => return false,
    };

    // Check if there are more pages to update.
    let has_next_page = match &data.data {
        Some(data) => match &data.site_statistics {
            Some(site_statistics) => match &site_statistics.manga {
                Some(manga) => match &manga.page_info {
                    Some(page_info) => page_info.has_next_page.unwrap_or(false),
                    None => false,
                },
                None => false,
            },
            None => false,
        },
        None => false,
    };

    // Update the last page number based on whether there are more pages to update.
    if has_next_page {
        if update_anime {
            random_stats.anime_last_page += 1;
        } else {
            random_stats.manga_last_page += 1;
        }
    }

    has_next_page
}
