use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub applied_count: usize,
    pub elapsed: Duration,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub version: String,
    pub name: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMigration {
    pub version: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MigrationFile {
    pub version: String,
    pub name: String,
    pub direction: MigrationDirection,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MigrationDirection {
    Up,
    Down,
}

impl MigrationFile {
    pub fn parse_filename(filename: &str) -> Option<(String, String, MigrationDirection)> {
        let stem = filename.strip_suffix(".sql")?;
        let (rest, dir_str) = if let Some(r) = stem.strip_suffix(".up") {
            (r, MigrationDirection::Up)
        } else if let Some(r) = stem.strip_suffix(".down") {
            (r, MigrationDirection::Down)
        } else {
            return None;
        };

        let underscore_pos = rest.find('_')?;
        let version = &rest[..underscore_pos];
        let name = &rest[underscore_pos + 1..];

        if version.is_empty() || name.is_empty() {
            return None;
        }

        Some((version.to_string(), name.to_string(), dir_str))
    }

    pub fn checksum(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_up_migration() {
        let result = MigrationFile::parse_filename("20240101000001_create_users.up.sql");
        assert!(result.is_some());
        let (version, name, dir) = result.unwrap();
        assert_eq!(version, "20240101000001");
        assert_eq!(name, "create_users");
        assert_eq!(dir, MigrationDirection::Up);
    }

    #[test]
    fn test_parse_down_migration() {
        let result = MigrationFile::parse_filename("20240101000001_create_users.down.sql");
        assert!(result.is_some());
        let (version, name, dir) = result.unwrap();
        assert_eq!(version, "20240101000001");
        assert_eq!(name, "create_users");
        assert_eq!(dir, MigrationDirection::Down);
    }

    #[test]
    fn test_parse_invalid_filename() {
        assert!(MigrationFile::parse_filename("invalid.sql").is_none());
        assert!(MigrationFile::parse_filename("no_direction.sql").is_none());
        assert!(MigrationFile::parse_filename("_.up.sql").is_none());
    }

    #[test]
    fn test_checksum_deterministic() {
        let content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
        let c1 = MigrationFile::checksum(content);
        let c2 = MigrationFile::checksum(content);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_checksum_differs_for_different_content() {
        let c1 = MigrationFile::checksum("CREATE TABLE users;");
        let c2 = MigrationFile::checksum("CREATE TABLE orders;");
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_migration_report_defaults() {
        let report = MigrationReport {
            applied_count: 0,
            elapsed: Duration::from_secs(0),
            errors: vec![],
        };
        assert_eq!(report.applied_count, 0);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_pending_migration() {
        let pending = PendingMigration {
            version: "20240101000001".to_string(),
            name: "create_users".to_string(),
        };
        assert_eq!(pending.version, "20240101000001");
        assert_eq!(pending.name, "create_users");
    }

    #[test]
    fn test_migration_status_with_applied_at() {
        let status = MigrationStatus {
            version: "20240101000001".to_string(),
            name: "create_users".to_string(),
            applied_at: Some(Utc::now()),
            checksum: "abc123".to_string(),
        };
        assert!(status.applied_at.is_some());
    }

    #[test]
    fn test_migration_status_without_applied_at() {
        let status = MigrationStatus {
            version: "20240101000001".to_string(),
            name: "create_users".to_string(),
            applied_at: None,
            checksum: "abc123".to_string(),
        };
        assert!(status.applied_at.is_none());
    }
}
