use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("{0}")]
    Gone(String),
    #[error("too many requests")]
    TooManyRequests,
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl From<sea_orm::TransactionError<sea_orm::DbErr>> for AppError {
    fn from(e: sea_orm::TransactionError<sea_orm::DbErr>) -> Self {
        match e {
            sea_orm::TransactionError::Connection(e) => AppError::from(e),
            sea_orm::TransactionError::Transaction(e) => AppError::from(e),
        }
    }
}

impl From<sea_orm::TransactionError<AppError>> for AppError {
    fn from(e: sea_orm::TransactionError<AppError>) -> Self {
        match e {
            sea_orm::TransactionError::Connection(e) => AppError::from(e),
            sea_orm::TransactionError::Transaction(e) => e,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            Self::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            Self::Gone(m) => (StatusCode::GONE, m.clone()),
            Self::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "Too many requests".into()),
            Self::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            Self::Internal(e) => {
                tracing::error!("Internal error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
        };
        tracing::error!("error: {}", message);
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    fn status(err: AppError) -> StatusCode {
        err.into_response().status()
    }

    #[test]
    fn not_found_maps_to_404() {
        assert_eq!(
            status(AppError::NotFound("x".into())),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn conflict_maps_to_409() {
        assert_eq!(status(AppError::Conflict("x".into())), StatusCode::CONFLICT);
    }

    #[test]
    fn bad_request_maps_to_400() {
        assert_eq!(
            status(AppError::BadRequest("x".into())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn unauthorized_maps_to_401() {
        assert_eq!(status(AppError::Unauthorized), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn gone_maps_to_410() {
        assert_eq!(status(AppError::Gone("x".into())), StatusCode::GONE);
    }

    #[test]
    fn too_many_requests_maps_to_429() {
        assert_eq!(
            status(AppError::TooManyRequests),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn database_error_maps_to_500() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        assert_eq!(
            status(AppError::Database(db_err)),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn internal_error_maps_to_500() {
        let err = anyhow::anyhow!("test internal error");
        assert_eq!(
            status(AppError::Internal(err)),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn error_response_body_contains_error_key() {
        use axum::body::to_bytes;
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let resp = AppError::NotFound("not here".into()).into_response();
                let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert!(json.get("error").is_some());
                assert_eq!(json["error"].as_str().unwrap(), "not here");
            });
    }
}
