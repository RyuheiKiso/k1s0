//! コンセンサス設定。
//!
//! YAML ファイルからデシリアライズされる設定構造体。

use serde::Deserialize;

/// コンセンサス機能の統合設定。
#[derive(Debug, Clone, Deserialize)]
pub struct ConsensusConfig {
    /// リーダー選出設定。
    #[serde(default)]
    pub leader: LeaderConfig,

    /// 分散ロック設定。
    #[serde(default)]
    pub lock: LockConfig,

    /// Saga 設定。
    #[serde(default)]
    pub saga: SagaConfig,

    /// Redis 設定（Redis バックエンド使用時）。
    pub redis: Option<RedisConfig>,
}

/// リーダー選出設定。
#[derive(Debug, Clone, Deserialize)]
pub struct LeaderConfig {
    /// リースキー。
    #[serde(default = "default_lease_key")]
    pub lease_key: String,

    /// リース有効期間（秒）。
    #[serde(default = "default_lease_duration_secs")]
    pub lease_duration_secs: u64,

    /// ハートビート間隔（秒）。リース有効期間の 1/3 程度を推奨。
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,

    /// リーダー変更の監視ポーリング間隔（秒）。
    #[serde(default = "default_watch_poll_interval_secs")]
    pub watch_poll_interval_secs: u64,
}

impl Default for LeaderConfig {
    fn default() -> Self {
        Self {
            lease_key: default_lease_key(),
            lease_duration_secs: default_lease_duration_secs(),
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
            watch_poll_interval_secs: default_watch_poll_interval_secs(),
        }
    }
}

/// 分散ロック設定。
#[derive(Debug, Clone, Deserialize)]
pub struct LockConfig {
    /// ロック取得のデフォルトタイムアウト（ミリ秒）。
    #[serde(default = "default_lock_timeout_ms")]
    pub default_timeout_ms: u64,

    /// ロック取得のポーリング間隔（ミリ秒）。
    #[serde(default = "default_lock_poll_interval_ms")]
    pub poll_interval_ms: u64,

    /// ロックのデフォルト TTL（秒）。
    #[serde(default = "default_lock_ttl_secs")]
    pub default_ttl_secs: u64,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: default_lock_timeout_ms(),
            poll_interval_ms: default_lock_poll_interval_ms(),
            default_ttl_secs: default_lock_ttl_secs(),
        }
    }
}

/// Saga 設定。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SagaConfig {
    /// デフォルトのリトライポリシー。
    #[serde(default)]
    pub retry: BackoffConfig,

    /// デッドレター設定。
    #[serde(default)]
    pub dead_letter: DeadLetterConfig,

    /// コレオグラフィ設定。
    #[serde(default)]
    pub choreography: ChoreographyConfig,
}

/// バックオフ設定。
#[derive(Debug, Clone, Deserialize)]
pub struct BackoffConfig {
    /// バックオフ戦略（`fixed`, `exponential`）。
    #[serde(default = "default_backoff_strategy")]
    pub strategy: String,

    /// 基本待機時間（ミリ秒）。
    #[serde(default = "default_backoff_base_ms")]
    pub base_ms: u64,

    /// 最大待機時間（ミリ秒）。
    #[serde(default = "default_backoff_max_ms")]
    pub max_ms: u64,

    /// 最大リトライ回数。
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            strategy: default_backoff_strategy(),
            base_ms: default_backoff_base_ms(),
            max_ms: default_backoff_max_ms(),
            max_retries: default_max_retries(),
        }
    }
}

/// Redis 接続設定。
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis 接続 URL。
    pub url: String,

    /// キープレフィックス。
    #[serde(default = "default_redis_key_prefix")]
    pub key_prefix: String,

    /// パスワードファイルパス（機密情報はファイル参照）。
    pub password_file: Option<String>,
}

/// デッドレター設定。
#[derive(Debug, Clone, Deserialize)]
pub struct DeadLetterConfig {
    /// デッドレターの最大保持件数。
    #[serde(default = "default_dead_letter_max_count")]
    pub max_count: u64,

    /// デッドレターの保持期間（日）。
    #[serde(default = "default_dead_letter_retention_days")]
    pub retention_days: u32,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            max_count: default_dead_letter_max_count(),
            retention_days: default_dead_letter_retention_days(),
        }
    }
}

/// コレオグラフィ Saga 設定。
#[derive(Debug, Clone, Deserialize)]
pub struct ChoreographyConfig {
    /// ステップタイムアウト（秒）。
    #[serde(default = "default_choreography_timeout_secs")]
    pub step_timeout_secs: u64,
}

impl Default for ChoreographyConfig {
    fn default() -> Self {
        Self {
            step_timeout_secs: default_choreography_timeout_secs(),
        }
    }
}

// --- デフォルト値関数 ---

fn default_lease_key() -> String {
    "k1s0-leader".into()
}

const fn default_lease_duration_secs() -> u64 {
    30
}

const fn default_heartbeat_interval_secs() -> u64 {
    10
}

const fn default_watch_poll_interval_secs() -> u64 {
    5
}

const fn default_lock_timeout_ms() -> u64 {
    10_000
}

const fn default_lock_poll_interval_ms() -> u64 {
    100
}

const fn default_lock_ttl_secs() -> u64 {
    30
}

fn default_backoff_strategy() -> String {
    "exponential".into()
}

const fn default_backoff_base_ms() -> u64 {
    100
}

const fn default_backoff_max_ms() -> u64 {
    10_000
}

const fn default_max_retries() -> u32 {
    3
}

fn default_redis_key_prefix() -> String {
    "k1s0:consensus:".into()
}

const fn default_dead_letter_max_count() -> u64 {
    1000
}

const fn default_dead_letter_retention_days() -> u32 {
    30
}

const fn default_choreography_timeout_secs() -> u64 {
    300
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_leader_config() {
        let config = LeaderConfig::default();
        assert_eq!(config.lease_duration_secs, 30);
        assert_eq!(config.heartbeat_interval_secs, 10);
    }

    #[test]
    fn test_default_lock_config() {
        let config = LockConfig::default();
        assert_eq!(config.default_timeout_ms, 10_000);
        assert_eq!(config.default_ttl_secs, 30);
    }

    #[test]
    fn test_default_saga_config() {
        let config = SagaConfig::default();
        assert_eq!(config.retry.max_retries, 3);
        assert_eq!(config.retry.strategy, "exponential");
    }

    #[test]
    fn test_deserialize_minimal() {
        let yaml = r"
leader:
  lease_key: my-service
lock: {}
saga: {}
";
        let config: ConsensusConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.leader.lease_key, "my-service");
        assert_eq!(config.leader.lease_duration_secs, 30);
    }
}
