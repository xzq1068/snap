use crate::db::error::{DbError, DbResult, is_unique_constraint};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDefinition {
    pub id: i64,
    pub project_id: i64,
    pub api_code: String,
    pub api_name: String,
    pub method: String,
    pub url: String,
    pub headers_json: Option<String>,
    pub query_json: Option<String>,
    pub body_json: Option<String>,
    pub auth_type: Option<String>,
    pub auth_config_json: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

pub(crate) async fn create_table(db_pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_definition (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            api_code TEXT NOT NULL,
            api_name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            headers_json TEXT,
            query_json TEXT,
            body_json TEXT,
            auth_type TEXT,
            auth_config_json TEXT,
            create_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            update_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES project_space(id) ON DELETE RESTRICT,
            UNIQUE(project_id, api_code)
        );
        "#,
    )
    .execute(db_pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_api_definition_project_id
        ON api_definition(project_id);
        "#,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(Clone)]
pub struct ApiDefinitionRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct NewApiDefinition<'a> {
    pub project_id: i64,
    pub api_code: &'a str,
    pub api_name: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub headers_json: Option<&'a str>,
    pub query_json: Option<&'a str>,
    pub body_json: Option<&'a str>,
    pub auth_type: Option<&'a str>,
    pub auth_config_json: Option<&'a str>,
}

impl ApiDefinitionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, definition: NewApiDefinition<'_>) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO api_definition (
                project_id,
                api_code,
                api_name,
                method,
                url,
                headers_json,
                query_json,
                body_json,
                auth_type,
                auth_config_json
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(definition.project_id)
        .bind(definition.api_code)
        .bind(definition.api_name)
        .bind(definition.method)
        .bind(definition.url)
        .bind(definition.headers_json)
        .bind(definition.query_json)
        .bind(definition.body_json)
        .bind(definition.auth_type)
        .bind(definition.auth_config_json)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) if is_unique_constraint(&error) => Err(DbError::Conflict {
                entity: "api_definition",
                field: "api_code",
                value: format!("{}:{}", definition.project_id, definition.api_code),
            }),
            Err(error) => Err(DbError::Database(error)),
        }
    }

    pub async fn delete(&self, id: i64) -> DbResult<()> {
        let result = sqlx::query("delete from api_definition where id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(DbError::Database)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "api_definition",
                id,
            });
        }

        Ok(())
    }

    pub async fn select_by_id(&self, id: i64) -> DbResult<ApiDefinition> {
        let result =
            sqlx::query_as::<_, ApiDefinition>("select * from api_definition where id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(DbError::Database)?;

        result.ok_or(DbError::NotFound {
            entity: "api_definition",
            id,
        })
    }

    pub async fn select_by_project_id(&self, project_id: i64) -> DbResult<Vec<ApiDefinition>> {
        let result = sqlx::query_as::<_, ApiDefinition>(
            r#"
            select *
            from api_definition
            where project_id = ?
            order by update_time desc, id desc
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Database)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::project_space::{
        ProjectSpaceRepository, create_table as create_project_space_table,
    };

    async fn test_repository() -> ApiDefinitionRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_project_space_table(&pool).await.unwrap();
        create_table(&pool).await.unwrap();

        let project_repository = ProjectSpaceRepository::new(pool.clone());
        project_repository
            .insert("Demo Project", "demo")
            .await
            .unwrap();

        ApiDefinitionRepository::new(pool)
    }

    fn new_definition<'a>(project_id: i64, api_code: &'a str) -> NewApiDefinition<'a> {
        NewApiDefinition {
            project_id,
            api_code,
            api_name: "Get User",
            method: "GET",
            url: "/users/{id}",
            headers_json: Some(r#"{"Authorization":"Bearer token"}"#),
            query_json: Some(r#"{"include":"profile"}"#),
            body_json: None,
            auth_type: Some("bearer"),
            auth_config_json: Some(r#"{"token":"demo-token"}"#),
        }
    }

    #[tokio::test]
    async fn insert_persists_api_definition_fields() {
        let repository = test_repository().await;

        repository
            .insert(new_definition(1, "get_user"))
            .await
            .unwrap();
        let definition = repository.select_by_id(1).await.unwrap();

        assert_eq!(definition.project_id, 1);
        assert_eq!(definition.api_code, "get_user");
        assert_eq!(definition.api_name, "Get User");
        assert_eq!(definition.method, "GET");
        assert_eq!(definition.url, "/users/{id}");
    }

    #[tokio::test]
    async fn duplicate_api_code_in_same_project_returns_conflict_error() {
        let repository = test_repository().await;

        repository
            .insert(new_definition(1, "get_user"))
            .await
            .unwrap();
        let error = repository
            .insert(NewApiDefinition {
                api_name: "Get User Again",
                ..new_definition(1, "get_user")
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            DbError::Conflict { entity, field, value }
                if entity == "api_definition"
                    && field == "api_code"
                    && value == "1:get_user"
        ));
    }

    #[tokio::test]
    async fn same_api_code_can_exist_in_different_projects() {
        let repository = test_repository().await;
        let project_repository = ProjectSpaceRepository::new(repository.pool.clone());

        project_repository
            .insert("Another Project", "another")
            .await
            .unwrap();

        repository
            .insert(new_definition(1, "get_user"))
            .await
            .unwrap();
        repository
            .insert(new_definition(2, "get_user"))
            .await
            .unwrap();

        let project_one_definitions = repository.select_by_project_id(1).await.unwrap();
        let project_two_definitions = repository.select_by_project_id(2).await.unwrap();

        assert_eq!(project_one_definitions.len(), 1);
        assert_eq!(project_two_definitions.len(), 1);
    }
}
