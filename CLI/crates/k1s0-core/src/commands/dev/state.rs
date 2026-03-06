/// 状態管理モジュール。
///
/// `.k1s0-dev/state.json` でローカル開発環境の起動状態を管理する。
use anyhow::Result;
use std::path::Path;

use super::types::DevState;

/// 状態ファイルのパス。
const STATE_FILE: &str = ".k1s0-dev/state.json";

/// 状態ファイルを読み込む。
///
/// ファイルが存在しない場合やパースに失敗した場合は `None` を返す。
pub fn load_state() -> Option<DevState> {
    load_state_from(Path::new(STATE_FILE))
}

/// 指定パスから状態ファイルを読み込む（テスト用）。
pub fn load_state_from(path: &Path) -> Option<DevState> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 状態ファイルに保存する。
///
/// # Errors
///
/// ファイルの書き込みに失敗した場合にエラーを返す。
pub fn save_state(state: &DevState) -> Result<()> {
    save_state_to(state, Path::new(STATE_FILE))
}

/// 指定パスに状態ファイルを保存する（テスト用）。
pub fn save_state_to(state: &DevState, path: &Path) -> Result<()> {
    // 親ディレクトリがなければ作成
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::dev::types::{DevStateDeps, KafkaDep, MigrationStatus, PostgresDep, RedisDep, SeedStatus};
    use tempfile::TempDir;

    /// 存在しないファイルからの読み込みは None を返す。
    #[test]
    fn test_load_state_nonexistent() {
        let state = load_state_from(Path::new("/nonexistent/state.json"));
        assert!(state.is_none());
    }

    /// 状態を保存して読み込めることを確認する。
    #[test]
    fn test_save_and_load_state() {
        let tmp = TempDir::new().unwrap();
        let state_path = tmp.path().join("state.json");

        let mut migration_status = std::collections::HashMap::new();
        migration_status.insert(
            "k1s0_system".to_string(),
            MigrationStatus { applied: 3, total: 3 },
        );
        let mut seed_status = std::collections::HashMap::new();
        seed_status.insert("k1s0_system".to_string(), SeedStatus { applied: true });

        let state = DevState {
            version: 1,
            started_at: "2026-03-06T00:00:00Z".to_string(),
            services: vec!["regions/system/server/rust/auth".to_string()],
            dependencies: DevStateDeps {
                postgres: Some(PostgresDep {
                    port: 5432,
                    databases: vec!["k1s0_system".to_string()],
                }),
                kafka: Some(KafkaDep { port: 9092 }),
                redis: Some(RedisDep { port: 6379 }),
            },
            auth_mode: "skip".to_string(),
            migration_status,
            seed_status,
        };

        save_state_to(&state, &state_path).unwrap();

        let loaded = load_state_from(&state_path).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.started_at, "2026-03-06T00:00:00Z");
        assert_eq!(loaded.services.len(), 1);
        assert_eq!(loaded.auth_mode, "skip");

        let pg = loaded.dependencies.postgres.unwrap();
        assert_eq!(pg.port, 5432);
        assert_eq!(pg.databases, vec!["k1s0_system"]);

        let kafka = loaded.dependencies.kafka.unwrap();
        assert_eq!(kafka.port, 9092);

        let redis = loaded.dependencies.redis.unwrap();
        assert_eq!(redis.port, 6379);
    }

    /// 依存なしの状態を保存・読み込みできることを確認する。
    #[test]
    fn test_save_and_load_state_no_deps() {
        let tmp = TempDir::new().unwrap();
        let state_path = tmp.path().join("state.json");

        let state = DevState {
            version: 1,
            started_at: "2026-03-06T12:00:00Z".to_string(),
            services: vec![],
            dependencies: DevStateDeps::default(),
            auth_mode: "keycloak".to_string(),
            migration_status: std::collections::HashMap::new(),
            seed_status: std::collections::HashMap::new(),
        };

        save_state_to(&state, &state_path).unwrap();

        let loaded = load_state_from(&state_path).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.auth_mode, "keycloak");
        assert!(loaded.dependencies.postgres.is_none());
        assert!(loaded.dependencies.kafka.is_none());
        assert!(loaded.dependencies.redis.is_none());
    }

    /// 不正な JSON からの読み込みは None を返す。
    #[test]
    fn test_load_state_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let state_path = tmp.path().join("state.json");
        std::fs::write(&state_path, "{invalid json").unwrap();

        let state = load_state_from(&state_path);
        assert!(state.is_none());
    }

    /// 親ディレクトリが存在しなくても保存できることを確認する。
    #[test]
    fn test_save_state_creates_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let state_path = tmp.path().join("nested").join("dir").join("state.json");

        let state = DevState {
            version: 1,
            started_at: "2026-03-06T00:00:00Z".to_string(),
            services: vec![],
            dependencies: DevStateDeps::default(),
            auth_mode: "skip".to_string(),
            migration_status: std::collections::HashMap::new(),
            seed_status: std::collections::HashMap::new(),
        };

        save_state_to(&state, &state_path).unwrap();
        assert!(state_path.exists());
    }
}
