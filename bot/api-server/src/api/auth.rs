use axum::{
	extract::State,
	http::Request,
	middleware::Next,
	response::Response,
};
use jsonwebtoken::{decode, Validation};
use tracing::warn;

use crate::api::error::AppError;
use crate::api::state::AppState;

pub use crate::api::oauth::Claims;

pub async fn auth_middleware(
	State(state): State<AppState>, mut req: Request<axum::body::Body>, next: Next,
) -> Result<Response, AppError> {
	let token = req
		.headers()
		.get("Authorization")
		.and_then(|value| value.to_str().ok())
		.and_then(|value| value.strip_prefix("Bearer "))
		.ok_or(AppError::unauthorized())?;

	let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

	let decoded_token = decode::<Claims>(token, &state.jwt_decoding_key, &validation)
		.map_err(|e| {
			warn!(error = ?e, "jwt validation failed");
			AppError::unauthorized()
		})?;

	req.extensions_mut().insert(decoded_token.claims);

	Ok(next.run(req).await)
}
