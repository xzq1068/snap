use crate::db::project_space::{ProjectSpace, ProjectSpaceRepositoryError};
use crate::server::{ApiResponse, ApiResult, AppError, AppState};
use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct CreateProjectSpaceRequest {
    pub project_name: String,
    pub project_code: String,
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(request): Json<CreateProjectSpaceRequest>,
) -> ApiResult<()> {
    if request.project_name.trim().is_empty() {
        return Err(AppError::BadRequest("project_name 不能为空".to_string()));
    }

    if request.project_code.trim().is_empty() {
        return Err(AppError::BadRequest("project_code 不能为空".to_string()));
    }

    match state
        .project_repository
        .insert(&request.project_name, &request.project_code)
        .await
    {
        Ok(_) => Ok(ApiResponse::success(None)),
        Err(e) => {
            error!("Failed to insert project_space: {:?}", e);
            Err(map_repository_error(e))
        }
    }
}

pub async fn delete_project(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    match state.project_repository.delete(id).await {
        Ok(_) => Ok(ApiResponse::success(None)),
        Err(e) => {
            error!("Failed to delete project_space: {:?}", e);
            Err(map_repository_error(e))
        }
    }
}

pub async fn select_all(State(state): State<AppState>) -> ApiResult<Vec<ProjectSpace>> {
    match state.project_repository.select_all().await {
        Ok(data) => Ok(ApiResponse::success(Some(data))),
        Err(e) => {
            error!("Failed to select all projects: {:?}", e);
            Err(map_repository_error(e))
        }
    }
}

pub async fn select_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<ProjectSpace> {
    match state.project_repository.select_by_id(id).await {
        Ok(data) => Ok(ApiResponse::success(Some(data))),
        Err(e) => {
            error!("Failed to select_by_id project_space: {:?}", e);
            Err(map_repository_error(e))
        }
    }
}

fn map_repository_error(error: ProjectSpaceRepositoryError) -> AppError {
    match error {
        ProjectSpaceRepositoryError::AlreadyExists(project_code) => {
            AppError::Conflict(format!("project_code `{project_code}` 已存在"))
        }
        ProjectSpaceRepositoryError::NotFound(id) => {
            AppError::NotFound(format!("id 为 {id} 的项目不存在"))
        }
        ProjectSpaceRepositoryError::Database(_) => AppError::InnerError,
    }
}
