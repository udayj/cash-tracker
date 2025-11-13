use libsql::{Builder, Connection};
use std::env;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Error building init database:{0}")]
    DatabaseBuildError(String),

    #[error("Database connection error: {0}")]
    ConnectionError(String),
}

pub struct DatabaseService {
    pub conn: Connection,
}

impl DatabaseService {
    pub async fn new(db_url: String) -> Result<Self, DatabaseError> {
        let url = db_url;
        let token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

        let db = Builder::new_remote(url, token)
            .build()
            .await
            .map_err(|e| DatabaseError::DatabaseBuildError(e.to_string()))?;
        let conn = db
            .connect()
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        Ok(Self { conn })
    }
}
