use crate::config::Config;
use crate::database::prelude::*;
use crate::event_handler::{BotData, RootUsage};
use anyhow::{Context as AnyhowContext, Result, anyhow};
use axum::{
	Router,
	extract::{Query, State},
	http::{HeaderMap, StatusCode},
	response::Json,
	routing::get,
};
use sea_orm::{
	ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
	QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info};

// Models for API responses
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
	success: bool,
	data: Option<T>,
	error: Option<String>,
}

#[derive(Debug, Serialize)]
struct CountResponse {
	count: i64,
}

#[derive(Debug, Serialize)]
struct AnimeResponseSong {
	anilist_id: String,
	anime_en_name: String,
	anime_jp_name: String,
	song_type: String,
	song_name: String,
	hq_url: String,
	mq_url: String,
	audio_url: String,
}

#[derive(Debug, Serialize)]
struct RandomStatsResponse {
	anime_last_page: i32,
	manga_last_page: i32,
}

#[derive(Debug, Serialize)]
struct CommandUsageResponse {
	command_name: String,
	usage_count: u128,
}

#[derive(Debug, Serialize)]
struct PingResponse {
	shard_id: String,
	latency: String,
	timestamp: String,
}

#[derive(Debug, Deserialize)]
struct ApiQuery {
	limit: Option<u64>,
	offset: Option<u64>,
}

// State that will be shared with all routes
#[derive(Clone)]
struct AppState {
	db: Arc<DatabaseConnection>,
	number_of_command_use_per_command: Arc<RwLock<RootUsage>>,
	api_key: String,
}

// Helper function to verify API key
fn verify_api_key(headers: &HeaderMap, expected_key: &str) -> bool {
	match headers.get("X-API-Key") {
		Some(key) => match key.to_str() {
			Ok(key_str) => key_str == expected_key,
			Err(_) => false,
		},
		None => false,
	}
}

// Start the API server
pub async fn start_api_server(config: Arc<Config>, bot_data: Arc<BotData<'_>>) -> Result<()> {
	// Get API config, return early if API is not enabled
	let api_config = if config.api.enabled {
		config.api.clone()
	} else {
		return Err(anyhow!("API disabled"));
	};

	// Create app state
	let app_state = AppState {
		db: bot_data.db_connection.clone(),
		number_of_command_use_per_command: bot_data.number_of_command_use_per_command.clone(),
		api_key: api_config.api_key.clone(),
	};

	// Create router with all our routes
	let app = Router::new()
		.route("/health", get(health_check))
		.route("/anime/songs", get(get_anime_songs))
		.route("/stats/random", get(get_random_stats))
		.route("/commands/usage", get(get_command_usage))
		.route("/commands/list", get(get_command_list))
		.route("/stats/ping", get(get_ping))
		.route("/stats/users", get(get_user_count))
		.route("/stats/guilds", get(get_guild_count))
		.with_state(app_state);

	// Run the server
	let addr = SocketAddr::from(([0, 0, 0, 0], api_config.port));
	info!("Starting API server at {}", addr);

	let listener = TcpListener::bind(&addr)
		.await
		.context("Failed to bind to address")?;

	axum::serve(listener, app)
		.await
		.context("Failed to start API server")?;

	Ok(())
}

// Health check endpoint - no auth required
async fn health_check() -> StatusCode {
	StatusCode::OK
}

// Get anime songs
async fn get_anime_songs(
	headers: HeaderMap, State(state): State<AppState>, Query(params): Query<ApiQuery>,
) -> Result<Json<ApiResponse<Vec<AnimeResponseSong>>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	let limit = params.limit.unwrap_or(20);
	let offset = params.offset.unwrap_or(0);

	// Query database
	match AnimeSong::find()
		.limit(Some(limit))
		.offset(Some(offset))
		.all(&*state.db)
		.await
	{
		Ok(songs) => {
			let response_songs = songs
				.into_iter()
				.map(|song| AnimeResponseSong {
					anilist_id: song.anilist_id,
					anime_en_name: song.anime_en_name,
					anime_jp_name: song.anime_jp_name,
					song_type: song.song_type,
					song_name: song.song_name,
					hq_url: song.hq,
					mq_url: song.mq,
					audio_url: song.audio,
				})
				.collect();

			Ok(Json(ApiResponse {
				success: true,
				data: Some(response_songs),
				error: None,
			}))
		},
		Err(err) => {
			error!("Failed to get anime songs: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

// Get random stats
async fn get_random_stats(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<RandomStatsResponse>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Query database
	match RandomStats::find().all(&*state.db).await {
		Ok(stats) => {
			if let Some(stat) = stats.first() {
				Ok(Json(ApiResponse {
					success: true,
					data: Some(RandomStatsResponse {
						anime_last_page: stat.last_anime_page,
						manga_last_page: stat.last_manga_page,
					}),
					error: None,
				}))
			} else {
				Ok(Json(ApiResponse {
					success: true,
					data: Some(RandomStatsResponse {
						anime_last_page: 0,
						manga_last_page: 0,
					}),
					error: None,
				}))
			}
		},
		Err(err) => {
			error!("Failed to get random stats: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

// Get command usage (aggregated, not per user)
async fn get_command_usage(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CommandUsageResponse>>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Get command usage from bot data
	let command_usage_lock = state.number_of_command_use_per_command.read().await;
	let command_list = &command_usage_lock.command_list;

	let mut response = Vec::new();

	for (command_name, user_info) in command_list {
		let mut total_usage: u128 = 0;

		for (_, user_usage) in &user_info.user_info {
			total_usage += user_usage.usage;
		}

		response.push(CommandUsageResponse {
			command_name: command_name.clone(),
			usage_count: total_usage,
		});
	}

	Ok(Json(ApiResponse {
		success: true,
		data: Some(response),
		error: None,
	}))
}

// Get command list
async fn get_command_list(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Query database
	match CommandList::find().all(&*state.db).await {
		Ok(commands) => {
			let command_names = commands.into_iter().map(|cmd| cmd.command_name).collect();

			Ok(Json(ApiResponse {
				success: true,
				data: Some(command_names),
				error: None,
			}))
		},
		Err(err) => {
			error!("Failed to get command list: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

// Get latest ping information
async fn get_ping(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PingResponse>>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Get latest ping per shard
	let result = PingHistory::find()
		.order_by_desc(crate::database::ping_history::Column::Timestamp)
		.all(&*state.db)
		.await;

	match result {
		Ok(pings) => {
			// Get latest ping per shard
			let mut latest_pings: HashMap<String, PingResponse> = HashMap::new();

			for ping in pings {
				if !latest_pings.contains_key(&ping.shard_id) {
					latest_pings.insert(
						ping.shard_id.clone(),
						PingResponse {
							shard_id: ping.shard_id,
							latency: ping.latency,
							timestamp: ping.timestamp.to_string(),
						},
					);
				}
			}

			Ok(Json(ApiResponse {
				success: true,
				data: Some(latest_pings.into_values().collect()),
				error: None,
			}))
		},
		Err(err) => {
			error!("Failed to get ping data: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

// Get user count
async fn get_user_count(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<CountResponse>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Count users
	match UserData::find().count(&*state.db).await {
		Ok(count) => {
			let count = count as i64;
			Ok(Json(ApiResponse {
				success: true,
				data: Some(CountResponse { count }),
				error: None,
			}))
		},
		Err(err) => {
			error!("Failed to get user count: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

// Get guild count
async fn get_guild_count(
	headers: HeaderMap, State(state): State<AppState>,
) -> Result<Json<ApiResponse<CountResponse>>, StatusCode> {
	// Verify API key
	if !verify_api_key(&headers, &state.api_key) {
		return Err(StatusCode::UNAUTHORIZED);
	}

	// Count guilds
	match GuildData::find().count(&*state.db).await {
		Ok(count) => {
			let count = count as i64;
			Ok(Json(ApiResponse {
				success: true,
				data: Some(CountResponse { count }),
				error: None,
			}))
		},
		Err(err) => {
			error!("Failed to get guild count: {}", err);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
