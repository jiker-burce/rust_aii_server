use sqlx::types::chrono;
use sqlx::{MySql, MySqlPool, Pool};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseConnectionError {
    #[error("Invalid Connection String ")]
    InvalidConnectionString(dotenvy::Error),
    #[error("Unable to connect to the database. ")]
    ConnectionError(#[from] sqlx::Error),
}

pub struct Database {
    underlying: Pool<MySql>,
}

impl Database {
    pub async fn new() -> Result<Self, DatabaseConnectionError> {
        let database_url = dotenvy::var("DATABASE_URL")
            .map_err(DatabaseConnectionError::InvalidConnectionString)?;
        let pool: Pool<MySql> = MySqlPool::connect(&database_url)
            .await
            .map_err(DatabaseConnectionError::ConnectionError)?;
        Ok(Database { underlying: pool })
    }
    // Pool internally uses a Clone, so this is not an expensive operation.
    pub fn get_pool(&self) -> &Pool<MySql> {
        &self.underlying
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_database_connection_get() {
        tokio_test::block_on(async {
            let db = Database::new().await.expect("Database connection expected");
            let row: (
                i64,
                String,
                i64,
                String,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
            ) = sqlx::query_as("SELECT * FROM users where id = ?")
                .bind(1_i64)
                .fetch_one(db.get_pool())
                .await
                .expect("error occured ");

            println!("row: {:?}", row);

            assert_eq!(row.1, "bruce");
        });
    }
}
