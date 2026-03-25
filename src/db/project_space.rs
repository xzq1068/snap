use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug,Serialize,Deserialize, sqlx::FromRow)]
pub struct ProjectSpace {
    pub id: i64,
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

#[derive(Clone)]
pub struct ProjectSpaceRepository {
    pool: SqlitePool,
}

impl ProjectSpaceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, project_name: &str,project_code:&str) -> Result<()> {
        sqlx::query(
            r#"
        INSERT INTO project_space (project_code, project_name)
        VALUES (?, ?)
        "#,
        )
        .bind(project_name)
        .bind(project_code)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id:i64) -> Result<()> {

        //todo 后期要校验项目空间是否关联了其他东西，否则不允许删除

        sqlx::query("delete from project_space WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn select_by_id(&self,id:i64) -> Result<Option<ProjectSpace>> {
        let result=sqlx::query_as::<_,ProjectSpace>("select * from project_space where id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    pub async fn select_all(&self)->Result<Vec<ProjectSpace>> {

        let result=sqlx::query_as::<_, ProjectSpace>("select * from project_space")
            .fetch_all(&self.pool)
            .await?;

        Ok(result)
    }


}
