mod project_space;

use crate::db::project_space::ProjectSpaceRepository;
use crate::server::project_space::{create_project, delete_project, select_all, select_by_id};
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    project_repository: Arc<ProjectSpaceRepository>,
}

pub type ApiResult<T> = std::result::Result<ApiResponse<T>, AppError>;

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

impl AppState {
    pub fn new(project_repository: Arc<ProjectSpaceRepository>) -> Self {
        Self { project_repository }
    }
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub msg: Option<String>,
    pub success: bool,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: Option<T>) -> Self {
        Self {
            success: true,
            code: "200".to_string(),
            msg: None,
            data,
        }
    }

    pub fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            success: false,
            code: code.into(),
            msg: Some(msg.into()),
            data: None,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ApiResponse::<()>::error(self.code(), self.msg());
        (status, Json(body)).into_response()
    }
}

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { ApiResponse::success(Some("success")) }),
        )
        .route("/projects", post(create_project).get(select_all))
        .route("/projects/{id}", get(select_by_id).delete(delete_project))
        .with_state(state)
}

pub async fn start_server(state: AppState) -> Result<()> {
    let app = app_router(state);
    let addr = "0.0.0.0:10086";

    let tcp_listener = TcpListener::bind(addr)
        .await
        .expect("start tcp server failed");

    info!("start web server http://{}", addr);

    axum::serve(tcp_listener, app)
        .await
        .expect("start web server failed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::project_space::{ProjectSpaceRepository, create_table};
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use sqlx::SqlitePool;
    use tower::util::ServiceExt;

    async fn test_app() -> Router {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_table(&pool).await.unwrap();

        let repository = Arc::new(ProjectSpaceRepository::new(pool));
        app_router(AppState::new(repository))
    }

    #[tokio::test]
    async fn health_endpoint_returns_success_payload() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["success"], true);
        assert_eq!(payload["data"], "success");
    }
}
