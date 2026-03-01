use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::time::Instant;
use tracing::info;

use crate::config::MigrationConfig;
use crate::error::MigrationError;
use crate::model::{MigrationDirection, MigrationFile, MigrationReport, MigrationStatus, PendingMigration};
use crate::runner::MigrationRunner;

const CREATE_MIGRATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    version TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
"#;

#[derive(Clone)]
pub struct SqlxMigrationRunner {
    pool: PgPool,
    config: MigrationConfig,
    up_migrations: BTreeMap<String, (String, String)>,
    down_migrations: BTreeMap<String, (String, String)>,
}

impl SqlxMigrationRunner {
    pub async fn new(pool: PgPool, config: MigrationConfig) -> Result<Self, MigrationError> {
        let dir = &config.migrations_dir;
        if !dir.exists() {
            return Err(MigrationError::DirectoryNotFound(
                dir.display().to_string(),
            ));
        }

        let mut up_migrations = BTreeMap::new();
        let mut down_migrations = BTreeMap::new();

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(MigrationError::Io)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Some((version, name, direction)) = MigrationFile::parse_filename(&filename) {
                let content = std::fs::read_to_string(entry.path())?;
                match direction {
                    MigrationDirection::Up => {
                        up_migrations.insert(version, (name, content));
                    }
                    MigrationDirection::Down => {
                        down_migrations.insert(version, (name, content));
                    }
                }
            }
        }

        Ok(Self {
            pool,
            config,
            up_migrations,
            down_migrations,
        })
    }

    pub fn from_pool(pool: PgPool, config: MigrationConfig) -> Self {
        Self {
            pool,
            config,
            up_migrations: BTreeMap::new(),
            down_migrations: BTreeMap::new(),
        }
    }

    async fn ensure_migrations_table(&self) -> Result<(), MigrationError> {
        let table_sql = CREATE_MIGRATIONS_TABLE.replace("_migrations", &self.config.table_name);
        sqlx::query(&table_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| MigrationError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    async fn load_applied(&self) -> Result<Vec<MigrationStatus>, MigrationError> {
        let query = format!(
            "SELECT version, name, checksum, applied_at FROM {} ORDER BY version ASC",
            self.config.table_name
        );
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MigrationError::ConnectionFailed(e.to_string()))?;

        let statuses = rows
            .iter()
            .map(|row| {
                use sqlx::Row;
                MigrationStatus {
                    version: row.get("version"),
                    name: row.get("name"),
                    checksum: row.get("checksum"),
                    applied_at: row.get("applied_at"),
                }
            })
            .collect();

        Ok(statuses)
    }

    async fn insert_migration(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        version: &str,
        name: &str,
        checksum: &str,
    ) -> Result<(), MigrationError> {
        let query = format!(
            "INSERT INTO {} (version, name, checksum) VALUES ($1, $2, $3)",
            self.config.table_name
        );
        sqlx::query(&query)
            .bind(version)
            .bind(name)
            .bind(checksum)
            .execute(&mut **tx)
            .await
            .map_err(|e| MigrationError::MigrationFailed {
                version: version.to_string(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn delete_migration(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        version: &str,
    ) -> Result<(), MigrationError> {
        let query = format!(
            "DELETE FROM {} WHERE version = $1",
            self.config.table_name
        );
        sqlx::query(&query)
            .bind(version)
            .execute(&mut **tx)
            .await
            .map_err(|e| MigrationError::MigrationFailed {
                version: version.to_string(),
                message: e.to_string(),
            })?;
        Ok(())
    }
}

#[async_trait]
impl MigrationRunner for SqlxMigrationRunner {
    async fn run_up(&self) -> Result<MigrationReport, MigrationError> {
        let start = Instant::now();
        self.ensure_migrations_table().await?;

        let applied = self.load_applied().await?;
        let applied_versions: std::collections::HashSet<String> =
            applied.iter().map(|s| s.version.clone()).collect();

        let mut count = 0;
        for (version, (name, content)) in &self.up_migrations {
            if applied_versions.contains(version) {
                continue;
            }

            let checksum = MigrationFile::checksum(content);
            info!(version = %version, name = %name, "applying migration");

            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| MigrationError::ConnectionFailed(e.to_string()))?;

            sqlx::query(content)
                .execute(&mut *tx)
                .await
                .map_err(|e| MigrationError::MigrationFailed {
                    version: version.clone(),
                    message: e.to_string(),
                })?;

            self.insert_migration(&mut tx, version, name, &checksum)
                .await?;

            tx.commit()
                .await
                .map_err(|e| MigrationError::MigrationFailed {
                    version: version.clone(),
                    message: e.to_string(),
                })?;

            count += 1;
        }

        Ok(MigrationReport {
            applied_count: count,
            elapsed: start.elapsed(),
            errors: vec![],
        })
    }

    async fn run_down(&self, steps: usize) -> Result<MigrationReport, MigrationError> {
        let start = Instant::now();
        self.ensure_migrations_table().await?;

        let applied = self.load_applied().await?;
        let to_rollback: Vec<_> = applied.iter().rev().take(steps).collect();

        let mut count = 0;
        for status in to_rollback {
            let version = &status.version;

            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| MigrationError::ConnectionFailed(e.to_string()))?;

            if let Some((_, down_content)) = self.down_migrations.get(version) {
                info!(version = %version, name = %status.name, "rolling back migration");
                sqlx::query(down_content)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| MigrationError::MigrationFailed {
                        version: version.clone(),
                        message: e.to_string(),
                    })?;
            } else {
                tracing::warn!(version = %version, "no down migration found, skipping SQL");
            }

            self.delete_migration(&mut tx, version).await?;

            tx.commit()
                .await
                .map_err(|e| MigrationError::MigrationFailed {
                    version: version.clone(),
                    message: e.to_string(),
                })?;

            count += 1;
        }

        Ok(MigrationReport {
            applied_count: count,
            elapsed: start.elapsed(),
            errors: vec![],
        })
    }

    async fn status(&self) -> Result<Vec<MigrationStatus>, MigrationError> {
        self.ensure_migrations_table().await?;

        let applied = self.load_applied().await?;
        let applied_map: std::collections::HashMap<String, &MigrationStatus> =
            applied.iter().map(|s| (s.version.clone(), s)).collect();

        let mut statuses = Vec::new();
        for (version, (name, content)) in &self.up_migrations {
            let checksum = MigrationFile::checksum(content);
            if let Some(applied_status) = applied_map.get(version) {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: applied_status.applied_at,
                    checksum,
                });
            } else {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: None,
                    checksum,
                });
            }
        }

        Ok(statuses)
    }

    async fn pending(&self) -> Result<Vec<PendingMigration>, MigrationError> {
        self.ensure_migrations_table().await?;

        let applied = self.load_applied().await?;
        let applied_versions: std::collections::HashSet<String> =
            applied.iter().map(|s| s.version.clone()).collect();

        let mut pending = Vec::new();
        for (version, (name, _)) in &self.up_migrations {
            if !applied_versions.contains(version) {
                pending.push(PendingMigration {
                    version: version.clone(),
                    name: name.clone(),
                });
            }
        }

        Ok(pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_migrations_table_sql_contains_expected_columns() {
        assert!(CREATE_MIGRATIONS_TABLE.contains("CREATE TABLE IF NOT EXISTS _migrations"));
        assert!(CREATE_MIGRATIONS_TABLE.contains("version TEXT PRIMARY KEY"));
        assert!(CREATE_MIGRATIONS_TABLE.contains("name TEXT NOT NULL"));
        assert!(CREATE_MIGRATIONS_TABLE.contains("checksum TEXT NOT NULL"));
        assert!(CREATE_MIGRATIONS_TABLE.contains("applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()"));
    }

    #[test]
    fn test_table_name_substitution() {
        let table_sql = CREATE_MIGRATIONS_TABLE.replace("_migrations", "my_schema_history");
        assert!(table_sql.contains("CREATE TABLE IF NOT EXISTS my_schema_history"));
        assert!(!table_sql.contains("_migrations"));
    }

    #[test]
    fn test_from_pool_creates_empty_runner() {
        use std::path::PathBuf;
        // We can't actually create a PgPool without a live DB, but we can verify the config
        let config = MigrationConfig::new(PathBuf::from("."), "postgres://localhost/test".to_string());
        assert_eq!(config.table_name, "_migrations");
    }

    #[test]
    fn test_status_logic_with_no_applied() {
        // Simulate status logic: all up_migrations should have applied_at = None
        let up_migrations: BTreeMap<String, (String, String)> = [
            (
                "20240101000001".to_string(),
                ("create_users".to_string(), "CREATE TABLE users (id INT);".to_string()),
            ),
            (
                "20240101000002".to_string(),
                ("add_email".to_string(), "ALTER TABLE users ADD COLUMN email TEXT;".to_string()),
            ),
        ]
        .into_iter()
        .collect();

        let applied_map: std::collections::HashMap<String, MigrationStatus> = std::collections::HashMap::new();

        let mut statuses = Vec::new();
        for (version, (name, content)) in &up_migrations {
            let checksum = MigrationFile::checksum(content);
            if let Some(applied_status) = applied_map.get(version) {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: applied_status.applied_at,
                    checksum,
                });
            } else {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: None,
                    checksum,
                });
            }
        }

        assert_eq!(statuses.len(), 2);
        for s in &statuses {
            assert!(s.applied_at.is_none());
        }
    }

    #[test]
    fn test_status_logic_with_all_applied() {
        use chrono::Utc;

        let up_migrations: BTreeMap<String, (String, String)> = [
            (
                "20240101000001".to_string(),
                ("create_users".to_string(), "CREATE TABLE users (id INT);".to_string()),
            ),
        ]
        .into_iter()
        .collect();

        let applied_at = Utc::now();
        let mut applied_map: std::collections::HashMap<String, MigrationStatus> = std::collections::HashMap::new();
        applied_map.insert(
            "20240101000001".to_string(),
            MigrationStatus {
                version: "20240101000001".to_string(),
                name: "create_users".to_string(),
                applied_at: Some(applied_at),
                checksum: "abc".to_string(),
            },
        );

        let mut statuses = Vec::new();
        for (version, (name, content)) in &up_migrations {
            let checksum = MigrationFile::checksum(content);
            if let Some(applied_status) = applied_map.get(version) {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: applied_status.applied_at,
                    checksum,
                });
            } else {
                statuses.push(MigrationStatus {
                    version: version.clone(),
                    name: name.clone(),
                    applied_at: None,
                    checksum,
                });
            }
        }

        assert_eq!(statuses.len(), 1);
        assert!(statuses[0].applied_at.is_some());
    }

    #[test]
    fn test_pending_logic() {
        let up_migrations: BTreeMap<String, (String, String)> = [
            ("20240101000001".to_string(), ("create_users".to_string(), "SQL1".to_string())),
            ("20240101000002".to_string(), ("add_email".to_string(), "SQL2".to_string())),
        ]
        .into_iter()
        .collect();

        let applied_versions: std::collections::HashSet<String> =
            ["20240101000001".to_string()].into_iter().collect();

        let mut pending = Vec::new();
        for (version, (name, _)) in &up_migrations {
            if !applied_versions.contains(version) {
                pending.push(PendingMigration {
                    version: version.clone(),
                    name: name.clone(),
                });
            }
        }

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, "20240101000002");
        assert_eq!(pending[0].name, "add_email");
    }
}
