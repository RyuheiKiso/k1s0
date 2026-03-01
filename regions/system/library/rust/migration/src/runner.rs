use async_trait::async_trait;
use std::collections::BTreeMap;
use std::time::Instant;
use tracing::{info, warn};

use crate::config::MigrationConfig;
use crate::error::MigrationError;
use crate::model::{
    MigrationDirection, MigrationFile, MigrationReport, MigrationStatus, PendingMigration,
};

#[async_trait]
pub trait MigrationRunner: Send + Sync {
    async fn run_up(&self) -> Result<MigrationReport, MigrationError>;
    async fn run_down(&self, steps: usize) -> Result<MigrationReport, MigrationError>;
    async fn status(&self) -> Result<Vec<MigrationStatus>, MigrationError>;
    async fn pending(&self) -> Result<Vec<PendingMigration>, MigrationError>;
}

#[derive(Debug)]
pub struct InMemoryMigrationRunner {
    _config: MigrationConfig,
    up_migrations: BTreeMap<String, (String, String)>,
    down_migrations: BTreeMap<String, (String, String)>,
    applied: tokio::sync::Mutex<Vec<MigrationStatus>>,
}

impl InMemoryMigrationRunner {
    pub fn new(config: MigrationConfig) -> Result<Self, MigrationError> {
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
            _config: config,
            up_migrations,
            down_migrations,
            applied: tokio::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn from_migrations(
        config: MigrationConfig,
        up_sqls: Vec<(String, String, String)>,
        down_sqls: Vec<(String, String, String)>,
    ) -> Self {
        let mut up_migrations = BTreeMap::new();
        let mut down_migrations = BTreeMap::new();

        for (version, name, content) in up_sqls {
            up_migrations.insert(version, (name, content));
        }
        for (version, name, content) in down_sqls {
            down_migrations.insert(version, (name, content));
        }

        Self {
            _config: config,
            up_migrations,
            down_migrations,
            applied: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl MigrationRunner for InMemoryMigrationRunner {
    async fn run_up(&self) -> Result<MigrationReport, MigrationError> {
        let start = Instant::now();
        let mut applied = self.applied.lock().await;
        let applied_versions: std::collections::HashSet<String> =
            applied.iter().map(|s| s.version.clone()).collect();

        let mut count = 0;
        for (version, (name, content)) in &self.up_migrations {
            if applied_versions.contains(version) {
                continue;
            }

            let checksum = MigrationFile::checksum(content);
            info!(version = %version, name = %name, "applying migration");

            applied.push(MigrationStatus {
                version: version.clone(),
                name: name.clone(),
                applied_at: Some(chrono::Utc::now()),
                checksum,
            });
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
        let mut applied = self.applied.lock().await;
        let mut count = 0;
        let errors = Vec::new();

        for _ in 0..steps {
            if applied.is_empty() {
                break;
            }
            let removed = applied.pop().unwrap();
            if !self.down_migrations.contains_key(&removed.version) {
                warn!(version = %removed.version, "no down migration found, skipping");
            }
            info!(version = %removed.version, name = %removed.name, "rolling back migration");
            count += 1;
        }

        Ok(MigrationReport {
            applied_count: count,
            elapsed: start.elapsed(),
            errors,
        })
    }

    async fn status(&self) -> Result<Vec<MigrationStatus>, MigrationError> {
        let applied = self.applied.lock().await;
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
        let applied = self.applied.lock().await;
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
    use std::path::PathBuf;

    fn test_config() -> MigrationConfig {
        MigrationConfig::new(PathBuf::from("."), "memory://".to_string())
    }

    fn create_runner() -> InMemoryMigrationRunner {
        InMemoryMigrationRunner::from_migrations(
            test_config(),
            vec![
                (
                    "20240101000001".to_string(),
                    "create_users".to_string(),
                    "CREATE TABLE users (id INT);".to_string(),
                ),
                (
                    "20240101000002".to_string(),
                    "add_email".to_string(),
                    "ALTER TABLE users ADD COLUMN email TEXT;".to_string(),
                ),
                (
                    "20240201000001".to_string(),
                    "create_orders".to_string(),
                    "CREATE TABLE orders (id INT);".to_string(),
                ),
            ],
            vec![
                (
                    "20240101000001".to_string(),
                    "create_users".to_string(),
                    "DROP TABLE users;".to_string(),
                ),
                (
                    "20240101000002".to_string(),
                    "add_email".to_string(),
                    "ALTER TABLE users DROP COLUMN email;".to_string(),
                ),
                (
                    "20240201000001".to_string(),
                    "create_orders".to_string(),
                    "DROP TABLE orders;".to_string(),
                ),
            ],
        )
    }

    #[tokio::test]
    async fn test_run_up_applies_all() {
        let runner = create_runner();
        let report = runner.run_up().await.unwrap();
        assert_eq!(report.applied_count, 3);
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn test_run_up_idempotent() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let report = runner.run_up().await.unwrap();
        assert_eq!(report.applied_count, 0);
    }

    #[tokio::test]
    async fn test_run_down() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let report = runner.run_down(1).await.unwrap();
        assert_eq!(report.applied_count, 1);

        let pending = runner.pending().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].version, "20240201000001");
    }

    #[tokio::test]
    async fn test_run_down_multiple_steps() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let report = runner.run_down(2).await.unwrap();
        assert_eq!(report.applied_count, 2);

        let pending = runner.pending().await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_run_down_more_than_applied() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let report = runner.run_down(10).await.unwrap();
        assert_eq!(report.applied_count, 3);
    }

    #[tokio::test]
    async fn test_status_all_pending() {
        let runner = create_runner();
        let statuses = runner.status().await.unwrap();
        assert_eq!(statuses.len(), 3);
        for s in &statuses {
            assert!(s.applied_at.is_none());
        }
    }

    #[tokio::test]
    async fn test_status_after_apply() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let statuses = runner.status().await.unwrap();
        assert_eq!(statuses.len(), 3);
        for s in &statuses {
            assert!(s.applied_at.is_some());
        }
    }

    #[tokio::test]
    async fn test_pending_returns_unapplied() {
        let runner = create_runner();
        let pending = runner.pending().await.unwrap();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].version, "20240101000001");
        assert_eq!(pending[1].version, "20240101000002");
        assert_eq!(pending[2].version, "20240201000001");
    }

    #[tokio::test]
    async fn test_pending_empty_after_apply() {
        let runner = create_runner();
        runner.run_up().await.unwrap();
        let pending = runner.pending().await.unwrap();
        assert!(pending.is_empty());
    }

    #[test]
    fn test_directory_not_found() {
        let config = MigrationConfig::new(
            PathBuf::from("/nonexistent/path"),
            "memory://".to_string(),
        );
        let result = InMemoryMigrationRunner::new(config);
        assert!(result.is_err());
        match result.unwrap_err() {
            MigrationError::DirectoryNotFound(_) => {}
            other => panic!("expected DirectoryNotFound, got {:?}", other),
        }
    }
}
