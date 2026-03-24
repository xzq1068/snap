use anyhow::Result;
use axum::routing::get;
use axum::Router;
use sysinfo::System;
use tokio::net::TcpListener;
use tracing::info;
pub async fn start_server() -> Result<()> {
    let app = Router::new().route(
        "/health",
        get(|| async {
            "success";
        }),
    );

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
