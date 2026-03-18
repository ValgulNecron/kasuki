use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};

pub struct AppError {
	pub status: StatusCode,
	pub message: String,
}

impl AppError {
	pub fn unauthorized() -> Self {
		Self {
			status: StatusCode::UNAUTHORIZED,
			message: "Unauthorized".into(),
		}
	}

	pub fn forbidden(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::FORBIDDEN,
			message: msg.into(),
		}
	}

	pub fn not_found(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::NOT_FOUND,
			message: msg.into(),
		}
	}

	pub fn bad_request(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::BAD_REQUEST,
			message: msg.into(),
		}
	}

	pub fn rate_limited() -> Self {
		Self {
			status: StatusCode::TOO_MANY_REQUESTS,
			message: "Rate limited".into(),
		}
	}

	pub fn internal(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			message: msg.into(),
		}
	}

	pub fn bad_gateway(msg: impl Into<String>) -> Self {
		Self {
			status: StatusCode::BAD_GATEWAY,
			message: msg.into(),
		}
	}
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		(
			self.status,
			Json(serde_json::json!({"error": self.message})),
		)
			.into_response()
	}
}

impl From<sea_orm::DbErr> for AppError {
	fn from(err: sea_orm::DbErr) -> Self {
		tracing::error!(error = %err, "database error");
		Self::internal("Database error")
	}
}

impl From<anyhow::Error> for AppError {
	fn from(err: anyhow::Error) -> Self {
		tracing::error!(error = %err, "internal error");
		Self::internal(err.to_string())
	}
}
