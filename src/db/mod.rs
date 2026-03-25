pub mod project_space;

use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::PathBuf;

pub async fn init_db(app_path: &PathBuf) -> Result<SqlitePool> {

    let dn_path = app_path.join(".snap.db");

    let db_url = format!("sqlite:{}?mode=rwc", dn_path.display());

    //1. 初始化数据库连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    //2. 初始化数据库表

    //1. 项目空间表 项目code贯穿其他表
    project_space::create_table(&pool).await?;

    //2. 接口表

    //3. 压测计划表

    //4. 插件表

    //5。压测结果表

    Ok(pool)
}
