use std::error::Error;
use std::fmt::{Display, Formatter};

pub type DbResult<T> = std::result::Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    Conflict {
        entity: &'static str,
        field: &'static str,
        value: String,
    },
    NotFound {
        entity: &'static str,
        id: i64,
    },
    Database(sqlx::Error),
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Conflict {
                entity,
                field,
                value,
            } => write!(f, "{entity}.{field} `{value}` already exists"),
            DbError::NotFound { entity, id } => write!(f, "{entity} id `{id}` not found"),
            DbError::Database(error) => write!(f, "{error}"),
        }
    }
}

impl Error for DbError {}

pub fn is_unique_constraint(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(db_error) => db_error.is_unique_violation(),
        _ => false,
    }
}
