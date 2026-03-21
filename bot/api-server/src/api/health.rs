use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::ConnectionTrait;

use crate::api::state::AppState;

pub async fn health_check() -> impl IntoResponse {
	Json(serde_json::json!({
		"status": "ok",
		"service": "kasuki-api"
	}))
}

pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
	match state.db.execute_unprepared("SELECT 1").await {
		Ok(_) => (
			StatusCode::OK,
			Json(serde_json::json!({
				"status": "ok",
				"checks": { "db": "connected" }
			})),
		),
		Err(e) => {
			tracing::warn!(error = %e, "readiness check: database probe failed");
			(
				StatusCode::SERVICE_UNAVAILABLE,
				Json(serde_json::json!({
					"status": "degraded",
					"checks": { "db": "disconnected" }
				})),
			)
		},
	}
}
