#[cfg(test)]
mod tests {
	use crate::api::error::AppError;
	use crate::api::health::health_check;
	use axum::http::StatusCode;
	use axum::response::IntoResponse;

	#[test]
	fn test_app_error_unauthorized() {
		let err = AppError::unauthorized();
		assert_eq!(err.status, StatusCode::UNAUTHORIZED);
		assert_eq!(err.message, "Unauthorized");
	}

	#[test]
	fn test_app_error_not_found() {
		let err = AppError::not_found("User not found");
		assert_eq!(err.status, StatusCode::NOT_FOUND);
		assert_eq!(err.message, "User not found");
	}

	#[test]
	fn test_app_error_bad_request() {
		let err = AppError::bad_request("Invalid code");
		assert_eq!(err.status, StatusCode::BAD_REQUEST);
		assert_eq!(err.message, "Invalid code");
	}

	#[test]
	fn test_app_error_rate_limited() {
		let err = AppError::rate_limited();
		assert_eq!(err.status, StatusCode::TOO_MANY_REQUESTS);
		assert_eq!(err.message, "Rate limited");
	}

	#[test]
	fn test_app_error_internal() {
		let err = AppError::internal("something broke");
		assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(err.message, "something broke");
	}

	#[test]
	fn test_app_error_bad_gateway() {
		let err = AppError::bad_gateway("Discord down");
		assert_eq!(err.status, StatusCode::BAD_GATEWAY);
		assert_eq!(err.message, "Discord down");
	}

	#[test]
	fn test_app_error_into_response_status() {
		let err = AppError::unauthorized();
		let response = err.into_response();
		assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
	}

	#[test]
	fn test_app_error_not_found_into_response_status() {
		let err = AppError::not_found("gone");
		let response = err.into_response();
		assert_eq!(response.status(), StatusCode::NOT_FOUND);
	}

	#[test]
	fn test_db_error_conversion() {
		let db_err = sea_orm::DbErr::Custom("test error".into());
		let app_err: AppError = db_err.into();
		assert_eq!(app_err.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(app_err.message, "Database error");
	}

	#[tokio::test]
	async fn test_health_check() {
		let response = health_check().await.into_response();
		assert_eq!(response.status(), StatusCode::OK);
	}

	#[test]
	fn test_jwt_claims_serialization() {
		use crate::api::oauth::Claims;

		let claims = Claims {
			sub: "123456".into(),
			username: "testuser".into(),
			exp: 9999999999,
		};

		let json = serde_json::to_string(&claims).unwrap();
		assert!(json.contains("123456"));
		assert!(json.contains("testuser"));

		let decoded: Claims = serde_json::from_str(&json).unwrap();
		assert_eq!(decoded.sub, "123456");
		assert_eq!(decoded.username, "testuser");
	}

	#[test]
	fn test_user_info_skip_serializing() {
		use crate::api::oauth::UserInfo;

		let user = UserInfo {
			id: "123".into(),
			username: "test".into(),
			discriminator: "0001".into(),
			avatar: Some("abc123".into()),
			email: Some("test@example.com".into()),
		};

		let json = serde_json::to_string(&user).unwrap();
		assert!(!json.contains("discriminator"));
		assert!(!json.contains("email"));
		assert!(json.contains("\"id\""));
		assert!(json.contains("\"username\""));
		assert!(json.contains("\"avatar\""));
	}

	#[test]
	fn test_guild_skip_serializing() {
		use crate::api::oauth::Guild;

		let guild = Guild {
			id: "789".into(),
			name: "Test Server".into(),
			icon_hash: Some("hash123".into()),
			icon_url: Some("https://cdn.discordapp.com/icons/789/hash123.png".into()),
			owner: true,
			permissions: "123456".into(),
		};

		let json = serde_json::to_string(&guild).unwrap();
		assert!(!json.contains("icon_hash"));
		assert!(!json.contains("owner"));
		assert!(!json.contains("permissions"));
		assert!(json.contains("\"id\""));
		assert!(json.contains("\"name\""));
		assert!(json.contains("icon_url"));
	}

	#[test]
	fn test_rate_limiter_creation() {
		use crate::api::rate_limit::create_rate_limiter;

		let limiter = create_rate_limiter(10);
		assert!(limiter.check_key(&"127.0.0.1".to_string()).is_ok());
	}

	#[test]
	fn test_jwt_encode_decode_roundtrip() {
		use crate::api::oauth::Claims;
		use base64::{engine::general_purpose::STANDARD, Engine as _};
		use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

		let secret = STANDARD.encode(b"test-secret-key-for-unit-tests!!");
		let secret_bytes = STANDARD.decode(&secret).unwrap();

		let claims = Claims {
			sub: "user123".into(),
			username: "testuser".into(),
			exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(&secret_bytes),
		)
		.unwrap();

		let decoded = decode::<Claims>(
			&token,
			&DecodingKey::from_secret(&secret_bytes),
			&Validation::new(jsonwebtoken::Algorithm::HS256),
		)
		.unwrap();

		assert_eq!(decoded.claims.sub, "user123");
		assert_eq!(decoded.claims.username, "testuser");
	}

	#[test]
	fn test_expired_jwt_rejected() {
		use crate::api::oauth::Claims;
		use base64::{engine::general_purpose::STANDARD, Engine as _};
		use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

		let secret = STANDARD.encode(b"test-secret-key-for-unit-tests!!");
		let secret_bytes = STANDARD.decode(&secret).unwrap();

		let claims = Claims {
			sub: "user123".into(),
			username: "testuser".into(),
			exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(&secret_bytes),
		)
		.unwrap();

		let result = decode::<Claims>(
			&token,
			&DecodingKey::from_secret(&secret_bytes),
			&Validation::new(jsonwebtoken::Algorithm::HS256),
		);

		assert!(result.is_err());
	}
}
