use axum::{
	extract::{Path, Query, State},
	response::IntoResponse,
	routing::{delete, get, post, put},
	Extension, Json, Router,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::api::auth::{auth_middleware, Claims};
use crate::api::error::AppError;
use crate::api::oauth::Guild;
use crate::api::state::AppState;
use shared::database::prelude::{ActivityData, GuildData, GuildLang, ModuleActivation};
use shared::database::{activity_data, guild_lang, module_activation};
use shared::localization::{Loader, USABLE_LOCALES};

fn locale_to_db_code(locale: &str) -> &str {
	match locale {
		"en-US" => "en",
		"ja" => "jp",
		other => other,
	}
}

fn db_code_to_locale(code: &str) -> &str {
	match code {
		"en" => "en-US",
		"jp" => "ja",
		other => other,
	}
}

fn locale_to_display_name(locale: &str) -> &str {
	match locale {
		"en-US" => "English",
		"ja" => "Japanese",
		"de" => "German",
		"fr" => "French",
		other => other,
	}
}

fn is_valid_lang(code: &str) -> bool {
	let locale = db_code_to_locale(code);
	USABLE_LOCALES.locales().any(|l| l.to_string() == locale)
}

pub fn guild_router(state: AppState) -> Router {
	Router::new()
		.route("/admin", get(get_admin_guilds))
		.route("/langs", get(get_available_langs))
		.route("/{guild_id}/settings", get(get_guild_settings))
		.route("/{guild_id}/lang", put(update_guild_lang))
		.route("/{guild_id}/modules", put(update_guild_modules))
		.route("/{guild_id}/activities", get(get_guild_activities))
		.route("/{guild_id}/activities", post(add_guild_activity))
		.route(
			"/{guild_id}/activities/{anime_id}",
			delete(delete_guild_activity),
		)
		.layer(axum::middleware::from_fn_with_state(
			state.clone(),
			auth_middleware,
		))
		.with_state(state)
}

pub fn anilist_router(state: AppState) -> Router {
	Router::new()
		.route("/search", get(search_anilist))
		.layer(axum::middleware::from_fn_with_state(
			state.clone(),
			auth_middleware,
		))
		.with_state(state)
}

/// Check that the user is admin (permission bit 0x8) of the guild
/// AND that the bot is present in the guild (guild_data table has a record).
async fn verify_guild_admin(
	state: &AppState, claims: &Claims, guild_id: &str,
) -> Result<(), AppError> {
	let (_, guilds) = state
		.get_cached_user(&claims.sub)
		.await
		.ok_or_else(|| AppError::unauthorized())?;

	let guild = guilds
		.iter()
		.find(|g| g.id == guild_id)
		.ok_or_else(|| AppError::forbidden("You are not a member of this guild"))?;

	let permissions: u64 = guild
		.permissions
		.as_ref()
		.and_then(|p| p.parse().ok())
		.unwrap_or(0);

	if permissions & 0x8 == 0 {
		return Err(AppError::forbidden("You are not an admin of this guild"));
	}

	GuildData::find_by_id(guild_id.to_string())
		.one(&*state.db)
		.await?
		.ok_or_else(|| AppError::not_found("Bot is not present in this guild"))?;

	Ok(())
}

#[derive(Serialize)]
struct LangOption {
	code: String,
	name: String,
}

#[derive(Serialize)]
struct AdminGuild {
	id: String,
	name: String,
	icon_url: Option<String>,
}

#[derive(Serialize)]
struct GuildSettings {
	lang: Option<String>,
	modules: ModuleSettings,
	activities: Vec<ActivityInfo>,
}

#[derive(Serialize)]
struct ModuleSettings {
	ai_module: bool,
	anilist_module: bool,
	game_module: bool,
	anime_module: bool,
	vn_module: bool,
	level_module: bool,
	mini_game_module: bool,
}

#[derive(Serialize)]
struct ActivityInfo {
	anime_id: i32,
	name: String,
	episode: i32,
	delay: i32,
	image: String,
}

#[derive(Deserialize)]
struct UpdateLangRequest {
	lang: String,
}

#[derive(Deserialize)]
struct UpdateModulesRequest {
	ai_module: Option<bool>,
	anilist_module: Option<bool>,
	game_module: Option<bool>,
	anime_module: Option<bool>,
	vn_module: Option<bool>,
	level_module: Option<bool>,
	mini_game_module: Option<bool>,
}

#[derive(Deserialize)]
struct AddActivityRequest {
	anime_name: Option<String>,
	anime_id: Option<i32>,
	channel_id: String,
	delay: Option<i32>,
}

#[derive(Deserialize)]
struct AnilistSearchQuery {
	q: String,
}

#[derive(Deserialize)]
struct AnilistGraphqlResponse {
	data: Option<AnilistData>,
}

#[derive(Deserialize)]
struct AnilistData {
	#[serde(rename = "Page")]
	page: Option<AnilistPage>,
	#[serde(rename = "Media")]
	media: Option<AnilistMedia>,
}

#[derive(Deserialize)]
struct AnilistPage {
	media: Option<Vec<AnilistMedia>>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AnilistMedia {
	id: i32,
	title: Option<AnilistTitle>,
	#[serde(rename = "coverImage")]
	cover_image: Option<AnilistCoverImage>,
	#[serde(rename = "nextAiringEpisode")]
	next_airing_episode: Option<AnilistNextAiring>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AnilistTitle {
	english: Option<String>,
	romaji: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AnilistCoverImage {
	#[serde(rename = "extraLarge")]
	extra_large: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct AnilistNextAiring {
	#[serde(rename = "airingAt")]
	airing_at: i64,
	episode: i32,
}

#[axum::debug_handler]
async fn get_available_langs() -> impl IntoResponse {
	let langs: Vec<LangOption> = USABLE_LOCALES
		.locales()
		.map(|locale| {
			let locale_str = locale.to_string();
			LangOption {
				code: locale_to_db_code(&locale_str).to_string(),
				name: locale_to_display_name(&locale_str).to_string(),
			}
		})
		.collect();
	Json(langs)
}

#[axum::debug_handler]
async fn get_admin_guilds(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
	let (_, guilds) = state
		.get_cached_user(&claims.sub)
		.await
		.ok_or_else(|| AppError::unauthorized())?;

	let admin_guild_ids: Vec<&Guild> = guilds
		.iter()
		.filter(|g| {
			let perms: u64 = g
				.permissions
				.as_ref()
				.and_then(|p| p.parse().ok())
				.unwrap_or(0);
			perms & 0x8 != 0
		})
		.collect();

	let mut result = Vec::new();
	for guild in admin_guild_ids {
		let exists = GuildData::find_by_id(guild.id.clone())
			.one(&*state.db)
			.await?;
		if exists.is_some() {
			result.push(AdminGuild {
				id: guild.id.clone(),
				name: guild.name.clone(),
				icon_url: guild.icon_url.clone(),
			});
		}
	}

	Ok(Json(result))
}

#[axum::debug_handler]
async fn get_guild_settings(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path(guild_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	let lang = GuildLang::find_by_id(guild_id.clone())
		.one(&*state.db)
		.await?
		.map(|l| l.lang);

	let modules = ModuleActivation::find()
		.filter(module_activation::Column::GuildId.eq(guild_id.clone()))
		.one(&*state.db)
		.await?;

	let module_settings = match modules {
		Some(m) => ModuleSettings {
			ai_module: m.ai_module,
			anilist_module: m.anilist_module,
			game_module: m.game_module,
			anime_module: m.anime_module,
			vn_module: m.vn_module,
			level_module: m.level_module,
			mini_game_module: m.mini_game_module,
		},
		None => ModuleSettings {
			ai_module: true,
			anilist_module: true,
			game_module: true,
			anime_module: true,
			vn_module: true,
			level_module: false,
			mini_game_module: true,
		},
	};

	let activities = ActivityData::find()
		.filter(activity_data::Column::ServerId.eq(guild_id.clone()))
		.all(&*state.db)
		.await?
		.into_iter()
		.map(|a| ActivityInfo {
			anime_id: a.anime_id,
			name: a.name,
			episode: a.episode,
			delay: a.delay,
			image: a.image,
		})
		.collect();

	Ok(Json(GuildSettings {
		lang,
		modules: module_settings,
		activities,
	}))
}

#[axum::debug_handler]
async fn update_guild_lang(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path(guild_id): Path<String>, Json(body): Json<UpdateLangRequest>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	if !is_valid_lang(&body.lang) {
		let valid: Vec<String> = USABLE_LOCALES
			.locales()
			.map(|l| locale_to_db_code(&l.to_string()).to_string())
			.collect();
		return Err(AppError::bad_request(format!(
			"Invalid language. Valid options: {}",
			valid.join(", ")
		)));
	}

	GuildLang::insert(guild_lang::ActiveModel {
		guild_id: Set(guild_id.clone()),
		lang: Set(body.lang.clone()),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(guild_lang::Column::GuildId)
			.update_column(guild_lang::Column::Lang)
			.to_owned(),
	)
	.exec(&*state.db)
	.await?;

	debug!(guild = %guild_id, lang = %body.lang, "updated guild language");
	Ok(Json(serde_json::json!({"status": "ok"})))
}

#[axum::debug_handler]
async fn update_guild_modules(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path(guild_id): Path<String>, Json(body): Json<UpdateModulesRequest>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	let existing = ModuleActivation::find()
		.filter(module_activation::Column::GuildId.eq(guild_id.clone()))
		.one(&*state.db)
		.await?;

	match existing {
		None => {
			let model = module_activation::ActiveModel {
				guild_id: Set(guild_id.clone()),
				ai_module: Set(body.ai_module.unwrap_or(true)),
				anilist_module: Set(body.anilist_module.unwrap_or(true)),
				game_module: Set(body.game_module.unwrap_or(true)),
				anime_module: Set(body.anime_module.unwrap_or(true)),
				vn_module: Set(body.vn_module.unwrap_or(true)),
				level_module: Set(body.level_module.unwrap_or(false)),
				mini_game_module: Set(body.mini_game_module.unwrap_or(true)),
				updated_at: Set(Utc::now().naive_utc()),
			};
			ModuleActivation::insert(model)
				.on_conflict(
					sea_orm::sea_query::OnConflict::column(module_activation::Column::GuildId)
						.update_columns([
							module_activation::Column::AiModule,
							module_activation::Column::AnilistModule,
							module_activation::Column::GameModule,
							module_activation::Column::AnimeModule,
							module_activation::Column::VnModule,
							module_activation::Column::LevelModule,
							module_activation::Column::MiniGameModule,
							module_activation::Column::UpdatedAt,
						])
						.to_owned(),
				)
				.exec(&*state.db)
				.await?;
		},
		Some(mut row) => {
			if let Some(v) = body.ai_module {
				row.ai_module = v;
			}
			if let Some(v) = body.anilist_module {
				row.anilist_module = v;
			}
			if let Some(v) = body.game_module {
				row.game_module = v;
			}
			if let Some(v) = body.anime_module {
				row.anime_module = v;
			}
			if let Some(v) = body.vn_module {
				row.vn_module = v;
			}
			if let Some(v) = body.level_module {
				row.level_module = v;
			}
			if let Some(v) = body.mini_game_module {
				row.mini_game_module = v;
			}

			use sea_orm::ActiveModelTrait;
			let mut active: module_activation::ActiveModel = row.into();
			active.updated_at = Set(Utc::now().naive_utc());
			active.update(&*state.db).await?;
		},
	}

	debug!(guild = %guild_id, "updated guild modules");
	Ok(Json(serde_json::json!({"status": "ok"})))
}

#[axum::debug_handler]
async fn get_guild_activities(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path(guild_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	let activities: Vec<ActivityInfo> = ActivityData::find()
		.filter(activity_data::Column::ServerId.eq(guild_id))
		.all(&*state.db)
		.await?
		.into_iter()
		.map(|a| ActivityInfo {
			anime_id: a.anime_id,
			name: a.name,
			episode: a.episode,
			delay: a.delay,
			image: a.image,
		})
		.collect();

	Ok(Json(activities))
}

#[axum::debug_handler]
async fn add_guild_activity(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path(guild_id): Path<String>, Json(body): Json<AddActivityRequest>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	// Resolve anime via AniList
	let media = if let Some(anime_id) = body.anime_id {
		fetch_anilist_by_id(&state.http_client, anime_id).await?
	} else if let Some(ref anime_name) = body.anime_name {
		fetch_anilist_by_search(&state.http_client, anime_name).await?
	} else {
		return Err(AppError::bad_request(
			"Either anime_name or anime_id is required",
		));
	};

	let next_airing = media
		.next_airing_episode
		.as_ref()
		.ok_or_else(|| AppError::bad_request("This anime has no upcoming episodes"))?;

	// Check if activity already exists
	let exists = ActivityData::find()
		.filter(activity_data::Column::AnimeId.eq(media.id))
		.filter(activity_data::Column::ServerId.eq(guild_id.clone()))
		.one(&*state.db)
		.await?;

	if exists.is_some() {
		return Err(AppError::bad_request(
			"Activity already exists for this anime in this guild",
		));
	}

	let title = media
		.title
		.as_ref()
		.ok_or_else(|| AppError::internal("AniList returned no title for this anime"))?;
	let anime_name = title
		.english
		.as_deref()
		.or(title.romaji.as_deref())
		.unwrap_or("Unknown")
		.to_string();
	let trimmed_name = anime_name.chars().take(100).collect::<String>();

	// Fetch and resize cover image
	let image_base64 = if let Some(ref cover) = media.cover_image {
		if let Some(ref url) = cover.extra_large {
			match fetch_and_resize_image(&state.http_client, url).await {
				Ok(b64) => b64,
				Err(e) => {
					error!("Failed to fetch cover image: {}", e.message);
					String::new()
				},
			}
		} else {
			String::new()
		}
	} else {
		String::new()
	};

	let timestamp = DateTime::<Utc>::from_timestamp(next_airing.airing_at, 0)
		.unwrap_or_default()
		.naive_utc();

	let delay = body.delay.unwrap_or(0);

	ActivityData::insert(activity_data::ActiveModel {
		anime_id: Set(media.id),
		timestamp: Set(timestamp),
		server_id: Set(guild_id.clone()),
		webhook: Set(None),
		episode: Set(next_airing.episode),
		name: Set(trimmed_name.clone()),
		delay: Set(delay),
		image: Set(image_base64),
		channel_id: Set(Some(body.channel_id.clone())),
	})
	.exec(&*state.db)
	.await?;

	debug!(guild = %guild_id, anime = %trimmed_name, "added activity via API");

	Ok(Json(serde_json::json!({
		"status": "ok",
		"anime_id": media.id,
		"name": trimmed_name,
		"episode": next_airing.episode,
	})))
}

#[axum::debug_handler]
async fn delete_guild_activity(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
	Path((guild_id, anime_id)): Path<(String, i32)>,
) -> Result<impl IntoResponse, AppError> {
	verify_guild_admin(&state, &claims, &guild_id).await?;

	let result = ActivityData::delete(activity_data::ActiveModel {
		anime_id: Set(anime_id),
		server_id: Set(guild_id.clone()),
		..Default::default()
	})
	.exec(&*state.db)
	.await?;

	if result.rows_affected == 0 {
		return Err(AppError::not_found("Activity not found"));
	}

	debug!(guild = %guild_id, anime_id = anime_id, "deleted activity");
	Ok(Json(serde_json::json!({"status": "ok"})))
}

#[axum::debug_handler]
async fn search_anilist(
	State(state): State<AppState>, Query(query): Query<AnilistSearchQuery>,
) -> Result<impl IntoResponse, AppError> {
	let gql_query = r#"query ($search: String) { Page(perPage: 10) { media(search: $search, type: ANIME) { id title { english romaji } coverImage { extraLarge } nextAiringEpisode { airingAt episode } } } }"#;

	let body = serde_json::json!({
		"query": gql_query,
		"variables": { "search": query.q }
	});

	let response = state
		.http_client
		.post("https://graphql.anilist.co")
		.json(&body)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("AniList API unreachable: {}", e)))?;

	if !response.status().is_success() {
		return Err(AppError::bad_gateway("AniList search failed"));
	}

	let gql_response: AnilistGraphqlResponse = response
		.json()
		.await
		.map_err(|e| AppError::bad_gateway(format!("Failed to parse AniList response: {}", e)))?;

	let results: Vec<AnilistMedia> = gql_response
		.data
		.and_then(|d| d.page)
		.and_then(|p| p.media)
		.unwrap_or_default();

	Ok(Json(results))
}

async fn fetch_anilist_by_search(
	client: &reqwest::Client, search: &str,
) -> Result<AnilistMedia, AppError> {
	let gql_query = r#"query ($search: String) { Media(search: $search, type: ANIME) { id title { english romaji } coverImage { extraLarge } nextAiringEpisode { airingAt episode } } }"#;

	let body = serde_json::json!({
		"query": gql_query,
		"variables": { "search": search }
	});

	let response = client
		.post("https://graphql.anilist.co")
		.json(&body)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("AniList API unreachable: {}", e)))?;

	let gql_response: AnilistGraphqlResponse = response
		.json()
		.await
		.map_err(|e| AppError::bad_gateway(format!("Failed to parse AniList response: {}", e)))?;

	gql_response
		.data
		.and_then(|d| d.media)
		.ok_or_else(|| AppError::not_found("Anime not found on AniList"))
}

async fn fetch_anilist_by_id(
	client: &reqwest::Client, anime_id: i32,
) -> Result<AnilistMedia, AppError> {
	let gql_query = r#"query ($id: Int) { Media(id: $id, type: ANIME) { id title { english romaji } coverImage { extraLarge } nextAiringEpisode { airingAt episode } } }"#;

	let body = serde_json::json!({
		"query": gql_query,
		"variables": { "id": anime_id }
	});

	let response = client
		.post("https://graphql.anilist.co")
		.json(&body)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("AniList API unreachable: {}", e)))?;

	let gql_response: AnilistGraphqlResponse = response
		.json()
		.await
		.map_err(|e| AppError::bad_gateway(format!("Failed to parse AniList response: {}", e)))?;

	gql_response
		.data
		.and_then(|d| d.media)
		.ok_or_else(|| AppError::not_found("Anime not found on AniList"))
}

async fn fetch_and_resize_image(client: &reqwest::Client, url: &str) -> Result<String, AppError> {
	let bytes = client
		.get(url)
		.send()
		.await
		.map_err(|e| AppError::bad_gateway(format!("Failed to fetch image: {}", e)))?
		.bytes()
		.await
		.map_err(|e| AppError::bad_gateway(format!("Failed to read image bytes: {}", e)))?;

	let resized = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, AppError> {
		let img = image::load_from_memory(&bytes)
			.map_err(|e| AppError::internal(format!("Failed to decode image: {}", e)))?;

		let (w, h) = (img.width(), img.height());
		let square_size = w.min(h);
		let crop_x = (w - square_size) / 2;
		let crop_y = (h - square_size) / 2;

		let cropped = img.crop_imm(crop_x, crop_y, square_size, square_size);
		let resized = cropped.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);

		let mut buf = std::io::Cursor::new(Vec::new());
		resized
			.write_to(&mut buf, image::ImageFormat::Jpeg)
			.map_err(|e| AppError::internal(format!("Failed to encode image: {}", e)))?;

		Ok(buf.into_inner())
	})
	.await
	.map_err(|e| AppError::internal(format!("Image processing task failed: {}", e)))??;

	let b64 = STANDARD.encode(&resized);
	Ok(format!("data:image/jpeg;base64,{}", b64))
}
