use std::sync::Arc;

use crate::{
    database_error::DatabaseError,
    models::{FileSet, FileType, PickedFileInfo, System},
};
use sqlx::{sqlite::SqliteRow, FromRow, Pool, Row, Sqlite};

pub struct SystemRepository {
    pool: Arc<Pool<Sqlite>>,
}

impl SystemRepository {
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }
    async fn get_system(&self, id: i64) -> Result<System, DatabaseError> {
        let system = sqlx::query_as!(
            System,
            "SELECT id, name 
             FROM system WHERE id = ?",
            id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(system)
    }

    async fn get_systems(&self) -> Result<Vec<System>, DatabaseError> {
        let systems = sqlx::query_as!(System, "SELECT id, name FROM system")
            .fetch_all(&*self.pool)
            .await?;
        Ok(systems)
    }

    async fn is_system_in_use(&self, system_id: i64) -> Result<bool, DatabaseError> {
        let releases_count = sqlx::query_scalar!(
            "SELECT COUNT(*) 
             FROM release_system 
             WHERE system_id = ?",
            system_id
        )
        .fetch_one(&*self.pool)
        .await?;

        let emulators_count = sqlx::query_scalar!(
            "SELECT COUNT(*) 
             FROM emulator_system 
             WHERE system_id = ?",
            system_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(releases_count > 0 || emulators_count > 0)
    }

    async fn add_system(&self, name: &String) -> Result<i64, DatabaseError> {
        let result = sqlx::query!("INSERT INTO system (name) VALUES (?)", name)
            .execute(&*self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
}

#[cfg(test)]
mod tests {
    use crate::setup_test_db;

    use super::*;
    use sqlx::query;

    const TEST_SYSTEM_NAME: &str = "Commodore 64";

    #[async_std::test]
    async fn test_get_system() {
        let pool = setup_test_db().await;
        let repo = SystemRepository {
            pool: Arc::new(pool),
        };
        let id = repo
            .add_system(&TEST_SYSTEM_NAME.to_string())
            .await
            .unwrap();
        let result = repo.get_system(id).await.unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.name, TEST_SYSTEM_NAME);
    }

    #[async_std::test]
    async fn test_get_systems() {
        let pool = setup_test_db().await;
        let repo = SystemRepository {
            pool: Arc::new(pool),
        };
        let id = repo
            .add_system(&TEST_SYSTEM_NAME.to_string())
            .await
            .unwrap();

        let result = repo.get_systems().await.unwrap();
        let result = &result[0];
        assert_eq!(result.id, id);
        assert_eq!(result.name, TEST_SYSTEM_NAME);
    }

    async fn insert_test_release(pool: &Pool<Sqlite>) -> i64 {
        let result = query!("INSERT INTO release (name) VALUES(?)", "Some release")
            .execute(pool)
            .await
            .unwrap();
        result.last_insert_rowid()
    }

    async fn insert_test_emulator(pool: &Pool<Sqlite>) -> i64 {
        let result = query!(
            "INSERT INTO emulator (name, executable, extract_files) VALUES(?,?,?)",
            "Test Emulator",
            "temu",
            false
        )
        .execute(pool)
        .await
        .unwrap();
        result.last_insert_rowid()
    }

    #[async_std::test]
    async fn test_is_system_in_use() {
        let pool = Arc::new(setup_test_db().await);
        let release_id = insert_test_release(&pool.clone()).await;
        let emulator_id = insert_test_emulator(&pool.clone()).await;

        let repo = SystemRepository { pool: pool.clone() };
        let system_id = repo
            .add_system(&TEST_SYSTEM_NAME.to_string())
            .await
            .unwrap();

        let result = repo.is_system_in_use(system_id).await.unwrap();
        assert!(!result);

        query!(
            "INSERT INTO release_system (release_id, system_id) VALUES (?, ?)",
            release_id,
            system_id
        )
        .execute(&*pool.clone())
        .await
        .unwrap();

        let result = repo.is_system_in_use(system_id).await.unwrap();
        assert!(result);

        query!("DELETE FROM release_system")
            .execute(&*pool.clone())
            .await
            .unwrap();

        let result = repo.is_system_in_use(system_id).await.unwrap();
        assert!(!result);

        query!(
            "INSERT INTO emulator_system (emulator_id, system_id) VALUES (?, ?)",
            emulator_id,
            system_id
        )
        .execute(&*pool.clone())
        .await
        .unwrap();

        let result = repo.is_system_in_use(system_id).await.unwrap();
        assert!(result);
    }
}
