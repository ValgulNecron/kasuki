use crate::constant::{RANDOM_STATS_PATH, TIME_BETWEEN_ACTIVITY_CHECK};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::info;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::site_statistic_anime::{AnimeStat, AnimeStatVariables};
use crate::structure::run::anilist::site_statistic_manga::{MangaStat, MangaStatVariables};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RandomStat {
    pub anime_last_page: i32,
    pub manga_last_page: i32,
}

pub async fn update_random_stats_launcher(anilist_cache: Arc<RwLock<Cache<String, String>>>) {
    info!("Starting random stats update");
    let mut interval = interval(Duration::from_secs(TIME_BETWEEN_ACTIVITY_CHECK));
    loop {
        interval.tick().await;
        let _ = update_random_stats(anilist_cache.clone()).await;
    }
}

pub async fn update_random_stats(
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat, AppError> {
    // try to load random stats from a json file
    let mut random_stats: RandomStat = match std::fs::read_to_string(RANDOM_STATS_PATH) {
        Ok(stats) => serde_json::from_str(&stats).map_err(|e| {
            AppError::new(
                format!("There was an error deserializing the random stats {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?,
        Err(_) => RandomStat {
            anime_last_page: 1796,
            manga_last_page: 1796,
        },
    };
    random_stats = update_random(random_stats, anilist_cache).await?;
    // write random stats to a json file
    let random_stats_json = serde_json::to_string(&random_stats).map_err(|e| {
        AppError::new(
            format!("There was an error serializing the random stats {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    std::fs::write(RANDOM_STATS_PATH, random_stats_json).map_err(|e| {
        AppError::new(
            format!(
                "There was an error writing the random stats to a file {}",
                e
            ),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    Ok(random_stats)
}

async fn update_random(
    mut random_stats: RandomStat,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<RandomStat, AppError> {
    let mut has_next_page = true;
    while has_next_page {
        let anime_page = random_stats.anime_last_page;
        let manga_page = random_stats.manga_last_page;
        let var = AnimeStatVariables {
            page: Some(anime_page),
        };
        let operation = AnimeStat::build(var);
        let data: Result<GraphQlResponse<AnimeStat>, AppError> =
            make_request_anilist(operation, false, anilist_cache.clone()).await;
        let data = data?;
        has_next_page = match data.data {
            Some(data) => match data.site_statistics {
                Some(site_statistics) => match site_statistics.manga {
                    Some(manga) => match manga.page_info {
                        Some(page_info) => page_info.has_next_page.unwrap_or(false),
                        None => false,
                    },
                    None => false,
                },
                None => false,
            },
            None => false,
        };
        if has_next_page {
            random_stats.anime_last_page = anime_page + 1;
            random_stats.manga_last_page = manga_page + 1;
        } else {
            random_stats.anime_last_page = anime_page - 1;
            random_stats.manga_last_page = manga_page - 1;
        }
    }

    let mut has_next_page = true;
    while has_next_page {
        let anime_page = random_stats.anime_last_page;
        let manga_page = random_stats.manga_last_page;
        let var = MangaStatVariables {
            page: Some(manga_page),
        };
        let operation = MangaStat::build(var);
        let data: Result<GraphQlResponse<AnimeStat>, AppError> =
            make_request_anilist(operation, false, anilist_cache.clone()).await;
        let data = data?;
        has_next_page = match data.data {
            Some(data) => match data.site_statistics {
                Some(site_statistics) => match site_statistics.manga {
                    Some(manga) => match manga.page_info {
                        Some(page_info) => page_info.has_next_page.unwrap_or(false),
                        None => false,
                    },
                    None => false,
                },
                None => false,
            },
            None => false,
        };
        if has_next_page {
            random_stats.anime_last_page = anime_page + 1;
            random_stats.manga_last_page = manga_page + 1;
        } else {
            random_stats.anime_last_page = anime_page - 1;
            random_stats.manga_last_page = manga_page - 1;
        }
    }

    Ok(random_stats)
}
