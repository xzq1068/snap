use crate::db::project_space::ProjectSpace;
use crate::server::{ApiResponse, AppError, AppState};
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct CreateProjectSpaceRequest {
    pub project_name: String,
    pub project_code: String,
}

pub async fn create_project(State(state): State<AppState>, Json(request): Json<CreateProjectSpaceRequest>) -> ApiResponse<()> {
    match state.project_repository.insert(&request.project_name, &request.project_code).await {
        Ok(_) => ApiResponse::success(None),
        Err(e) => {
            error!("Failed to insert project_space: {:?}", e);
            ApiResponse::from_error(AppError::InnerError)
        },
    }
}

pub async fn delete_project(State(state): State<AppState>, Path(id):Path<i64>) -> ApiResponse<()> {
    match state.project_repository.delete(id).await {
        Ok(_) => ApiResponse::success(None),
        Err(e) => {
            error!("Failed to delete project_space: {:?}", e);
            ApiResponse::from_error(AppError::InnerError)
        },
    }
}

pub async fn select_all(State(state): State<AppState>) -> ApiResponse<Vec<ProjectSpace>> {
    match state.project_repository.select_all().await {
        Ok(data) => {ApiResponse::success(Some(data))},
        Err(e) => {
            error!("Failed to select all projects: {:?}", e);
            ApiResponse::from_error(AppError::InnerError)
        }
    }
}

pub async fn select_by_id(State(state): State<AppState>, Path(id):Path<i64>) -> ApiResponse<ProjectSpace> {
    match state.project_repository.select_by_id(id).await {
        Ok(data)  => ApiResponse::success(data),
        Err(e)=>{
            error!("Failed to select_by_id project_space: {:?}", e);
            ApiResponse::from_error(AppError::InnerError)
        }
    }
}

