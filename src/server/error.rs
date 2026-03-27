use crate::db::error::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::server::ApiResponse;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Conflict(String),
    NotFound(String),
    InnerError,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "400",
            AppError::Conflict(_) => "409",
            AppError::NotFound(_) => "404",
            AppError::InnerError => "500",
        }
    }

    pub fn msg(&self) -> String {
        match self {
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::InnerError => "服务器内部错误".to_string(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InnerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<DbError> for AppError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::Conflict {
                entity,
                field,
                value,
            } => AppError::Conflict(format!("{entity}.{field} `{value}` 已存在")),
            DbError::NotFound { entity, id } => {
                AppError::NotFound(format!("{entity} id `{id}` 不存在"))
            }
            DbError::Database(_) => AppError::InnerError,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ApiResponse::<()>::error(self.code(), self.msg());
        (status, Json(body)).into_response()
    }
}
