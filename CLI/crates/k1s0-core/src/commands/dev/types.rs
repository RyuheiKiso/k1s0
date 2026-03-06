/// ローカル開発コマンドの型定義。
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 操作種別。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevOperation {
    /// 起動
    Up,
    /// 停止
    Down,
    /// 状態確認
    Status,
    /// ログ表示
    Logs,
}

/// 認証モード。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMode {
    /// スキップ（Keycloakなし、JWT検証無効）
    Skip,
    /// Keycloak（フル認証フロー）
    Keycloak,
}

/// クリーンアップレベル。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupLevel {
    /// コンテナのみ停止（ボリューム保持）
    ContainersOnly,
    /// コンテナとボリュームを削除
    ContainersAndVolumes,
}

/// dev up 設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevUpConfig {
    /// 選択されたサービスのパスリスト
    pub services: Vec<String>,
    /// 認証モード
    pub auth_mode: AuthMode,
}

/// dev down 設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevDownConfig {
    /// クリーンアップレベル
    pub cleanup: CleanupLevel,
}

/// 検出された依存情報。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectedDependencies {
    /// データベース依存のリスト
    pub databases: Vec<DatabaseDep>,
    /// Kafka を使用するか
    pub has_kafka: bool,
    /// Redis を使用するか
    pub has_redis: bool,
    /// Redis (session) を使用するか
    pub has_redis_session: bool,
    /// Kafka トピック一覧
    pub kafka_topics: Vec<String>,
}

/// データベース依存情報。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseDep {
    /// データベース名
    pub name: String,
    /// サービス名
    pub service: String,
}

/// ローカル開発環境の状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevState {
    /// 状態ファイルのバージョン
    pub version: u32,
    /// 起動日時 (RFC 3339)
    pub started_at: String,
    /// 対象サービスのパスリスト
    pub services: Vec<String>,
    /// 依存情報
    pub dependencies: DevStateDeps,
    /// 認証モード
    pub auth_mode: String,
    /// マイグレーション状態（DB名 → 状態）
    #[serde(default)]
    pub migration_status: std::collections::HashMap<String, MigrationStatus>,
    /// シードデータ状態（DB名 → 状態）
    #[serde(default)]
    pub seed_status: std::collections::HashMap<String, SeedStatus>,
}

/// マイグレーション状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// 適用済みの数
    pub applied: u32,
    /// 合計数
    pub total: u32,
}

/// シードデータ状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedStatus {
    /// 適用済みかどうか
    pub applied: bool,
}

/// 状態ファイルの依存情報。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevStateDeps {
    /// `PostgreSQL` 依存
    pub postgres: Option<PostgresDep>,
    /// Kafka 依存
    pub kafka: Option<KafkaDep>,
    /// Redis 依存
    pub redis: Option<RedisDep>,
}

/// `PostgreSQL` 依存情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDep {
    /// ポート番号
    pub port: u16,
    /// データベース名リスト
    pub databases: Vec<String>,
}

/// Kafka 依存情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaDep {
    /// ポート番号
    pub port: u16,
}

/// Redis 依存情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisDep {
    /// ポート番号
    pub port: u16,
}

/// ポート割り当て。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PortAssignments {
    /// `PostgreSQL` ポート
    pub postgres: u16,
    /// Kafka ポート
    pub kafka: u16,
    /// Redis ポート
    pub redis: u16,
    /// Redis (session) ポート
    pub redis_session: u16,
    /// pgAdmin ポート
    pub pgadmin: u16,
    /// Kafka UI ポート
    pub kafka_ui: u16,
    /// Keycloak ポート
    pub keycloak: u16,
}

/// シードファイルのパス。
#[derive(Debug, Clone)]
pub struct SeedFile {
    /// ファイルパス
    pub path: PathBuf,
    /// サービス名
    pub service: String,
}
