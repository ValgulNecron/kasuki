use axum::{
	extract::State,
	http::{Request, StatusCode},
	middleware::Next,
	response::{IntoResponse, Response},
	Json,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use shared::config::Config;
use std::sync::Arc;
use tracing::warn;

pub use crate::api::oauth::Claims;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub enum ApiError {
	Unauthorized,
	Forbidden,
	InternalServerError,
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		let (status, error_message) = match self {
			ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
			ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
			ApiError::InternalServerError => {
				(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
			},
		};

		(status, Json(serde_json::json!({"error": error_message}))).into_response()
	}
}

pub async fn auth_middleware(
	State(config): State<Arc<Config>>, mut req: Request<axum::body::Body>, next: Next,
) -> Result<Response, StatusCode> {
	let token = req
		.headers()
		.get("Authorization")
		.and_then(|value| value.to_str().ok())
		.and_then(|value| value.strip_prefix("Bearer "));

	let token = match token {
		Some(token) => token,
		None => return Err(StatusCode::UNAUTHORIZED),
	};

	let secret = STANDARD.decode(&config.api.oauth.jwt_secret).unwrap();
	let decoding_key = DecodingKey::from_secret(&secret);
	let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

	let decoded_token = match decode::<Claims>(token, &decoding_key, &validation) {
		Ok(token) => token,
		Err(e) => {
			warn!("JWT decoding error: {:?}", e);
			return Err(StatusCode::UNAUTHORIZED);
		},
	};

	req.extensions_mut().insert(decoded_token.claims);

	Ok(next.run(req).await)
}
