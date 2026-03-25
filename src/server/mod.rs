mod project_space;

use std::sync::Arc;
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sysinfo::System;
use tokio::net::TcpListener;
use tracing::info;
use crate::db::project_space::ProjectSpaceRepository;

#[derive(Clone)]
pub struct AppState{
    project_repository: Arc<ProjectSpaceRepository>
}

#[derive(Serialize, Debug)]
pub enum  AppError{
    InnerError
}

impl AppError{
    pub fn code(&self) -> String{
        match self {
            AppError::InnerError=> "500".to_string(),
        }
    }

    pub fn msg(&self)->String{
        match self {
            AppError::InnerError => "服务器内部错误".to_string(),
        }
    }
}

impl AppState{
    pub fn new(project_repository:Arc<ProjectSpaceRepository>)->Self{
        Self{project_repository }
    }
}

#[derive(Serialize)]
pub struct ApiResponse<T>{
    pub code:String,
    pub msg:Option<String>,
    pub success:bool,
    pub data:Option<T>
}

impl<T> ApiResponse<T>
where T: Serialize
{
    pub fn success(data: Option<T>)->Self {
        Self{
            success:true,
            code:"200".to_string(),
            msg: None,
            data
        }
    }

    pub fn error(msg:String)->Self{
        Self{
            success: false,
            code:"500".to_string(),
            msg: Some(msg),
            data: None
        }
    }

    pub fn from_error(error:AppError)->Self{
        Self{
            success: false,
            code: error.code(),
            msg: Some(error.msg()),
            data: None
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // 默认返回 200 OK，内容是我们的 JSON
        (StatusCode::OK, Json(self)).into_response()
    }
}


pub async fn start_server(state:AppState) -> Result<()> {
    let app = Router::new().route(
        "/health", get(|| async { "success"; }),
    ).with_state(state);

    let addr = "0.0.0.0:10086";

    let tcp_listener = TcpListener::bind(addr)
        .await
        .expect("start tcp server failed");

    let mut sys = System::new_all();
    sys.refresh_all();  // 刷新所有信息

    info!("🚀 start web server http://{}", addr);


    axum::serve(tcp_listener, app)
        .await
        .expect("start web server failed");

    Ok(())
}
