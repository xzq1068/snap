use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::error::Error;
use std::fmt::{Display, Formatter};

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

#[derive(Debug)]
pub enum ProjectSpaceRepositoryError {
    AlreadyExists(String),
    NotFound(i64),
    Database(sqlx::Error),
}

impl Display for ProjectSpaceRepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectSpaceRepositoryError::AlreadyExists(project_code) => {
                write!(f, "project_code `{project_code}` already exists")
            }
            ProjectSpaceRepositoryError::NotFound(id) => write!(f, "project id `{id}` not found"),
            ProjectSpaceRepositoryError::Database(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ProjectSpaceRepositoryError {}

impl ProjectSpaceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        project_name: &str,
        project_code: &str,
    ) -> std::result::Result<(), ProjectSpaceRepositoryError> {
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
            Err(error) if is_unique_constraint(&error) => Err(
                ProjectSpaceRepositoryError::AlreadyExists(project_code.to_string()),
            ),
            Err(error) => Err(ProjectSpaceRepositoryError::Database(error)),
        }
    }

    pub async fn delete(&self, id: i64) -> std::result::Result<(), ProjectSpaceRepositoryError> {
        // todo: 后期要校验项目空间是否关联了其他东西，否则不允许删除
        let result = sqlx::query("delete from project_space WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ProjectSpaceRepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(ProjectSpaceRepositoryError::NotFound(id));
        }

        Ok(())
    }

    pub async fn select_by_id(
        &self,
        id: i64,
    ) -> std::result::Result<ProjectSpace, ProjectSpaceRepositoryError> {
        let result = sqlx::query_as::<_, ProjectSpace>("select * from project_space where id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ProjectSpaceRepositoryError::Database)?;

        result.ok_or(ProjectSpaceRepositoryError::NotFound(id))
    }

    pub async fn select_all(
        &self,
    ) -> std::result::Result<Vec<ProjectSpace>, ProjectSpaceRepositoryError> {
        let result = sqlx::query_as::<_, ProjectSpace>(
            "select * from project_space order by update_time desc, id desc",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(ProjectSpaceRepositoryError::Database)?;

        Ok(result)
    }
}

fn is_unique_constraint(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(db_error) => {
            db_error.is_unique_violation()
                || db_error
                    .message()
                    .contains("UNIQUE constraint failed: project_space.project_code")
        }
        _ => false,
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
            ProjectSpaceRepositoryError::AlreadyExists(project_code) if project_code == "demo"
        ));
    }
}
