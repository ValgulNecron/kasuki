use axum::{extract::State, response::IntoResponse, Extension, Json};
use tracing::{debug, error};

use crate::api::auth::Claims;
use crate::api::error::AppError;
use crate::api::state::AppState;

#[axum::debug_handler]
pub async fn request_blacklist(
	State(state): State<AppState>, Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
	let webhook_url = state
		.config
		.api
		.blacklist_webhook_url
		.as_ref()
		.ok_or_else(|| AppError::internal("Blacklist webhook not configured"))?;

	let content = format!(
		"the user with id: {} want to be added to the blacklist",
		claims.sub
	);

	let payload = serde_json::json!({ "content": content });

	let response = state
		.http_client
		.post(webhook_url)
		.json(&payload)
		.send()
		.await
		.map_err(|e| {
			error!(error = %e, "failed to send blacklist webhook");
			AppError::bad_gateway("Failed to send blacklist request")
		})?;

	if !response.status().is_success() {
		let status = response.status();
		error!(status = %status, "blacklist webhook returned non-success");
		return Err(AppError::bad_gateway("Blacklist webhook request failed"));
	}

	debug!(user = %claims.sub, "blacklist request sent");
	Ok(Json(serde_json::json!({"status": "ok"})))
}
