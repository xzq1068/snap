use crate::db::error::{DbError, DbResult, is_unique_constraint};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

    pub async fn insert(&self, project_name: &str, project_code: &str) -> DbResult<()> {
        let result = sqlx::query(
            r#"
        INSERT INTO project_space (project_code, project_name)
        VALUES (?, ?)
        "#,
        )
        .bind(project_code)
        .bind(project_name)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) if is_unique_constraint(&error) => Err(DbError::Conflict {
                entity: "project_space",
                field: "project_code",
                value: project_code.to_string(),
            }),
            Err(error) => Err(DbError::Database(error)),
        }
    }

    pub async fn delete(&self, id: i64) -> DbResult<()> {
        // todo: 后期要校验项目空间是否关联了其他东西，否则不允许删除
        let result = sqlx::query("delete from project_space WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(DbError::Database)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "project_space",
                id,
            });
        }

        Ok(())
    }

    pub async fn select_by_id(&self, id: i64) -> DbResult<ProjectSpace> {
        let result = sqlx::query_as::<_, ProjectSpace>("select * from project_space where id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::Database)?;

        result.ok_or(DbError::NotFound {
            entity: "project_space",
            id,
        })
    }

    pub async fn select_all(&self) -> DbResult<Vec<ProjectSpace>> {
        let result = sqlx::query_as::<_, ProjectSpace>(
            "select * from project_space order by update_time desc, id desc",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Database)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_repository() -> ProjectSpaceRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_table(&pool).await.unwrap();
        ProjectSpaceRepository::new(pool)
    }

    #[tokio::test]
    async fn insert_persists_code_and_name_in_correct_columns() {
        let repository = test_repository().await;

        repository.insert("Demo Project", "demo").await.unwrap();
        let project = repository.select_by_id(1).await.unwrap();

        assert_eq!(project.project_name, "Demo Project");
        assert_eq!(project.project_code, "demo");
    }

    #[tokio::test]
    async fn duplicate_project_code_returns_conflict_error() {
        let repository = test_repository().await;

        repository.insert("Demo Project", "demo").await.unwrap();
        let error = repository
            .insert("Another Project", "demo")
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            DbError::Conflict { entity, field, value }
                if entity == "project_space" && field == "project_code" && value == "demo"
        ));
    }
}
