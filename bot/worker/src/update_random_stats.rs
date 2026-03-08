use anyhow::{Context, Result};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use cynic::{GraphQlResponse, QueryBuilder};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use shared::anilist::make_request::make_request_anilist;
use shared::anilist::site_statistic_anime::{AnimeStat, AnimeStatVariables};
use shared::anilist::site_statistic_manga::{MangaStat, MangaStatVariables};
use shared::cache::CacheInterface;
use shared::config::TaskIntervalConfig;
use shared::database::random_stats;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Clone, Copy)]
enum StatCategory {
	Anime,
	Manga,
}

impl fmt::Display for StatCategory {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Anime => write!(f, "anime"),
			Self::Manga => write!(f, "manga"),
		}
	}
}

struct PageState {
	anime_last_page: i32,
	manga_last_page: i32,
}

impl Default for PageState {
	fn default() -> Self {
		Self {
			anime_last_page: 1796,
			manga_last_page: 1796,
		}
	}
}

pub async fn update_random_stats_launcher(
	anilist_cache: Arc<CacheInterface>, task_intervals: TaskIntervalConfig,
	db: Arc<DatabaseConnection>, mut shutdown_rx: broadcast::Receiver<()>,
) {
	info!("Launching random stats update task");
	let mut interval =
		tokio::time::interval(Duration::from_secs(task_intervals.random_stats_update));

	const MAX_CONSECUTIVE_FAILURES: u32 = 3;
	let mut consecutive_failures = 0;
	let base_retry_delay = Duration::from_secs(5);
	let mut cycle = 0u64;

	loop {
		tokio::select! {
			_ = shutdown_rx.recv() => {
				info!("Random stats task received shutdown signal");
				break;
			}
			_ = interval.tick() => {
				cycle += 1;
				info!("Starting random stats update cycle #{}", cycle);

				match update_random_stats(anilist_cache.clone(), db.clone()).await {
					Ok(state) => {
						info!(
							"Cycle #{} done: anime_page={}, manga_page={}",
							cycle, state.anime_last_page, state.manga_last_page
						);
						if consecutive_failures > 0 {
							info!("Recovered after {} consecutive failures", consecutive_failures);
							consecutive_failures = 0;
						}
					},
					Err(err) => {
						consecutive_failures += 1;
						error!("Cycle #{} failed: {:#}", cycle, err);

						if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
							let delay = base_retry_delay
								.mul_f32(1.5_f32.powi(consecutive_failures as i32 - 3));
							warn!("Backing off for {:?} after {} failures", delay, consecutive_failures);
							sleep(delay).await;
						}
					},
				}
			}
		}
	}
}

async fn update_random_stats(
	anilist_cache: Arc<CacheInterface>, db: Arc<DatabaseConnection>,
) -> Result<PageState> {
	let mut state = match random_stats::Entity::find_by_id(1).one(&*db).await? {
		Some(model) => {
			debug!(
				"Loaded page state: anime={}, manga={}",
				model.last_anime_page, model.last_manga_page
			);
			PageState {
				anime_last_page: model.last_anime_page,
				manga_last_page: model.last_manga_page,
			}
		},
		None => {
			info!("No saved page state, using defaults");
			PageState::default()
		},
	};

	let anime_pages = update_category(StatCategory::Anime, &mut state, anilist_cache.clone()).await;
	info!("Anime stats: processed {} pages", anime_pages);

	let manga_pages = update_category(StatCategory::Manga, &mut state, anilist_cache).await;
	info!("Manga stats: processed {} pages", manga_pages);

	let active_model = random_stats::ActiveModel {
		id: Set(1),
		last_anime_page: Set(state.anime_last_page),
		last_manga_page: Set(state.manga_last_page),
	};

	random_stats::Entity::insert(active_model)
		.on_conflict(
			sea_orm::sea_query::OnConflict::column(random_stats::Column::Id)
				.update_columns([
					random_stats::Column::LastAnimePage,
					random_stats::Column::LastMangaPage,
				])
				.to_owned(),
		)
		.exec(&*db)
		.await
		.context("Failed to save page state to database")?;

	Ok(state)
}

/// Paginate through one category (anime or manga) until no more pages or too many failures.
async fn update_category(
	category: StatCategory, state: &mut PageState, anilist_cache: Arc<CacheInterface>,
) -> u32 {
	const MAX_FAILURES: u32 = 5;
	let mut failures = 0;
	let mut pages_processed = 0;

	info!(
		"Starting {} update from page {}",
		category,
		get_page(state, category)
	);

	loop {
		if failures >= MAX_FAILURES {
			warn!(
				"{} update stopped after {} failures",
				category, MAX_FAILURES
			);
			break;
		}

		let page = get_page(state, category);
		match fetch_page(category, page, anilist_cache.clone()).await {
			Ok(has_next) => {
				failures = 0;
				pages_processed += 1;

				if has_next {
					increment_page(state, category);

					if pages_processed % 10 == 0 {
						debug!("{} progress: {} pages processed", category, pages_processed);
					}
				} else {
					debug!("{} reached last page at {}", category, page);
					break;
				}
			},
			Err(e) => {
				failures += 1;
				warn!(
					"{} page {} failed ({}/{}): {:#}",
					category, page, failures, MAX_FAILURES, e
				);

				let delay = Duration::from_secs((2 * failures).into());
				sleep(delay).await;
				continue;
			},
		}

		sleep(Duration::from_secs(1)).await;
	}

	pages_processed
}

/// Fetch a single page of statistics from the AniList API.
async fn fetch_page(
	category: StatCategory, page: i32, anilist_cache: Arc<CacheInterface>,
) -> Result<bool> {
	let data: GraphQlResponse<AnimeStat> = match category {
		StatCategory::Anime => {
			let op = AnimeStat::build(AnimeStatVariables { page: Some(page) });
			make_request_anilist(op, true, anilist_cache).await?
		},
		StatCategory::Manga => {
			let op = MangaStat::build(MangaStatVariables { page: Some(page) });
			make_request_anilist(op, true, anilist_cache).await?
		},
	};

	let has_next = data
		.data
		.and_then(|d| d.site_statistics)
		.and_then(|s| s.manga)
		.and_then(|m| m.page_info)
		.and_then(|p| p.has_next_page)
		.unwrap_or(false);

	Ok(has_next)
}

fn get_page(state: &PageState, category: StatCategory) -> i32 {
	match category {
		StatCategory::Anime => state.anime_last_page,
		StatCategory::Manga => state.manga_last_page,
	}
}

fn increment_page(state: &mut PageState, category: StatCategory) {
	match category {
		StatCategory::Anime => state.anime_last_page += 1,
		StatCategory::Manga => state.manga_last_page += 1,
	}
}
