use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct ProjectSpace {
    pub id:  u64,
    pub project_code: String,
    pub project_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}



pub(crate) async fn create_table(db_pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS project_space (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_code TEXT NOT NULL UNIQUE,
            project_name TEXT NOT NULL,
            create_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            update_time DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
        .execute(db_pool)
        .await?;

    Ok(())
}


