pub mod db;
pub mod log;
pub mod server;

use crate::db::init_db;
use crate::db::project_space::ProjectSpaceRepository;
use crate::log::init_log;
use crate::server::{start_server, AppState};
use anyhow::Result;
use colored::Colorize;
use home::home_dir;
use std::fs::create_dir_all;
use std::sync::Arc;
use sysinfo::System;

#[tokio::main]
async fn main() -> Result<()> {
    let home_dir = home_dir().expect("Could not find home directory");

    let data_dir = home_dir.join(".snap");

    if !data_dir.exists() {
        create_dir_all(&data_dir)?;
    }

    //banner
    print_startup_banner();

    // 安装 sqlx Any 驱动（必须在连接数据库之前调用）
    // sqlx::any::install_default_drivers();  // 不再需要

    //1. 初始化日志
    let _guard = init_log(&data_dir).await?;

    //2. 初始化数据库
    let pool = init_db(&data_dir).await?;

    //3. 初始化web Server
    let project_space_repo = Arc::new(ProjectSpaceRepository::new(pool));

    let app_state = AppState::new(project_space_repo);

    start_server(app_state).await?;

    Ok(())
}

pub fn print_startup_banner() {
    let version = env!("CARGO_PKG_VERSION");

    println!();
    println!(
        "{}",
        r#"
   _____ _   _          _____
  / ____| \ | |   /\   |  __ \
 | (___ |  \| |  /  \  | |__) |
  \___ \| . ` | / /\ \ |  ___/
  ____) | |\  |/ ____ \| |
 |_____/|_| \_/_/    \_\_|

    "#
        .bright_cyan()
        .bold()
    );

    println!("  {} {}", "Version:".bright_green(), version);

    // 系统信息
    let mut sys = System::new_all();
    sys.refresh_all();

    println!();
    println!(
        "{}",
        "  ┌──────────────────────────────────────┐".bright_black()
    );
    println!(
        "  │ {} {}",
        "OS:".bright_blue(),
        format!("{:30}", System::name().unwrap_or_default())
    );
    println!(
        "  │ {} {}",
        "Kernel:".bright_blue(),
        format!("{:30}", System::kernel_version().unwrap_or_default())
    );
    println!(
        "  │ {} {}",
        "CPU:".bright_blue(),
        format!("{:30}", sys.cpus()[0].brand())
    );
    println!(
        "  │ {} {}",
        "Memory:".bright_blue(),
        format!(
            "{:.1}/{:.1} GB",
            sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
        )
    );
    println!(
        "{}",
        "  └──────────────────────────────────────┘".bright_black()
    );
    println!();
}
