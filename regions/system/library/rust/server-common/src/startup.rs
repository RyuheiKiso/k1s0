//! サーバー起動ボイラープレート削減のための `ServerBuilder` モジュール。
//!
//! 全サーバーで繰り返される初期化処理（テレメトリ・DB プール・JWKS 検証器・Metrics）を
//! 共通ビルダーパターンに抽出し、startup.rs の重複コードを最小化する。
//!
//! # 使用例
//!
//! ```rust,ignore
//! use k1s0_server_common::startup::ServerBuilder;
//!
//! let builder = ServerBuilder::new("k1s0-notification-server", "0.1.0", "system");
//! builder.init_telemetry(&cfg.app.environment, &cfg.observability)?;
//! let pool = builder.init_db_pool_from_config(&cfg.database).await?;
//! let jwks = builder.init_jwks_verifier(&cfg.auth)?;
//! let metrics = builder.create_metrics();
//! ```

use std::sync::Arc;
use tracing::info;

/// サーバー起動の共通初期化を一元管理するビルダー。
///
/// 各サーバーの startup.rs で繰り返されるボイラープレートを排除する。
/// テレメトリ初期化・DB プール作成・JWKS 検証器・Metrics 生成など、
/// 31 サーバーで共通する初期化処理を提供する。
pub struct ServerBuilder {
    /// サービス名（例: "k1s0-notification-server"）
    service_name: String,
    /// サービスバージョン（例: "0.1.0"）
    version: String,
    /// サービス階層（例: "system", "business"）
    tier: String,
}

impl ServerBuilder {
    /// 新しい ServerBuilder を生成する。
    /// service_name・version・tier は全初期化処理で共有される識別子。
    /// tier は "system" / "business" / "service" のいずれかを指定する（デフォルト値なし）。
    pub fn new(
        service_name: impl Into<String>,
        version: impl Into<String>,
        tier: impl Into<String>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            version: version.into(),
            tier: tier.into(),
        }
    }

    /// サービス名を取得する。
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// テレメトリ（トレーシング・ログ）を初期化する。
    ///
    /// 全サーバーで同一の TelemetryConfig 構築ロジックを共通化する。
    /// ObservabilityConfig の各フィールドから TelemetryConfig を組み立て、
    /// k1s0_telemetry::init_telemetry() を呼び出す。
    pub fn init_telemetry(
        &self,
        environment: &str,
        observability: &ObservabilityFields,
    ) -> anyhow::Result<()> {
        // テレメトリ設定を Observability 構成から構築する
        let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
            service_name: self.service_name.clone(),
            version: self.version.clone(),
            tier: self.tier.clone(),
            environment: environment.to_string(),
            trace_endpoint: observability
                .trace_enabled
                .then(|| observability.trace_endpoint.clone()),
            sample_rate: observability.sample_rate,
            log_level: observability.log_level.clone(),
            log_format: observability.log_format.clone(),
        };
        k1s0_telemetry::init_telemetry(&telemetry_cfg)
            .map_err(|e| anyhow::anyhow!("テレメトリ初期化に失敗: {}", e))?;
        Ok(())
    }

    /// Metrics インスタンスを生成する。
    /// service_name をラベルとして使用し、全メトリクスをサービス単位で識別する。
    pub fn create_metrics(&self) -> Arc<k1s0_telemetry::metrics::Metrics> {
        Arc::new(k1s0_telemetry::metrics::Metrics::new(&self.service_name))
    }

    /// PostgreSQL 接続プールを作成する（DatabasePoolConfig 指定）。
    ///
    /// 接続 URL の決定ロジック:
    /// 1. 環境変数 DATABASE_URL があればそちらを優先
    /// 2. なければ DatabasePoolConfig.url を使用
    ///
    /// 全サーバーで繰り返される PgPoolOptions 構築を一箇所に集約する。
    #[cfg(feature = "startup-db")]
    pub async fn init_db_pool(&self, config: &DatabasePoolConfig) -> anyhow::Result<sqlx::PgPool> {
        // DATABASE_URL 環境変数を優先する（Kubernetes Secret 注入対応）
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.url.clone());
        info!(
            service = %self.service_name,
            max_connections = config.max_connections,
            min_connections = config.min_connections,
            "データベース接続プールを作成中"
        );

        // PgPool を構築する（タイムアウト・接続数は設定値から取得）
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout_secs))
            .max_lifetime(config.max_lifetime_secs.map(std::time::Duration::from_secs))
            .connect(&url)
            .await?;

        info!(
            service = %self.service_name,
            "データベース接続プール確立完了"
        );
        Ok(pool)
    }

    /// 環境変数 DATABASE_URL のみから PostgreSQL 接続プールを作成する。
    ///
    /// 設定ファイルに database セクションが無い場合のフォールバック用。
    /// デフォルト値（max_connections=25, min_connections=5）で接続する。
    #[cfg(feature = "startup-db")]
    pub async fn init_db_pool_from_env(&self) -> anyhow::Result<Option<sqlx::PgPool>> {
        // DATABASE_URL が設定されていなければ None を返す
        match std::env::var("DATABASE_URL") {
            Ok(url) => {
                info!(
                    service = %self.service_name,
                    "DATABASE_URL からデータベース接続プールを作成中"
                );
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(25)
                    .min_connections(5)
                    .acquire_timeout(std::time::Duration::from_secs(30))
                    .max_lifetime(Some(std::time::Duration::from_secs(300)))
                    .connect(&url)
                    .await?;
                info!(
                    service = %self.service_name,
                    "DATABASE_URL からデータベース接続プール確立完了"
                );
                Ok(Some(pool))
            }
            Err(_) => Ok(None),
        }
    }

    /// JWKS トークン検証器を作成する。
    ///
    /// JwksAuthConfig（jwt + jwks）から JwksVerifier を構築する。
    /// 全サーバーで繰り返される JwksVerifier::new() の呼び出しを共通化する。
    #[cfg(feature = "startup-auth")]
    pub fn init_jwks_verifier(
        &self,
        auth_config: &JwksAuthConfig,
    ) -> anyhow::Result<Arc<k1s0_auth::JwksVerifier>> {
        // JWKS 設定からエンドポイント URL とキャッシュ TTL を取得する
        let jwks = auth_config
            .jwks
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("JWKS 設定が必要です（auth.jwks セクション）"))?;
        info!(
            service = %self.service_name,
            jwks_url = %jwks.url,
            "JWKS 検証器を初期化中"
        );
        // JwksVerifier を構築する（HTTP クライアント構築失敗時はエラー）
        let verifier = k1s0_auth::JwksVerifier::new(
            &jwks.url,
            &auth_config.jwt.issuer,
            &auth_config.jwt.audience,
            std::time::Duration::from_secs(jwks.cache_ttl_secs),
        )
        .map_err(|e| anyhow::anyhow!("JWKS 検証器の作成に失敗: {}", e))?;

        info!(
            service = %self.service_name,
            "JWKS 検証器の初期化完了"
        );
        Ok(Arc::new(verifier))
    }

    /// テレメトリをシャットダウンする。
    ///
    /// サーバー停止時にトレースエクスポーターをフラッシュし、
    /// 未送信のスパンを確実に送出する。
    pub fn shutdown_telemetry(&self) {
        info!(service = %self.service_name, "テレメトリをシャットダウン中");
        k1s0_telemetry::shutdown();
    }
}

/// テレメトリ初期化に必要な Observability 設定フィールド。
///
/// 各サーバーの ObservabilityConfig から変換して使用する。
/// サーバー固有の Config 構造体に依存しないよう、必要なフィールドのみを抽出した
/// 中間構造体として設計している。
#[derive(Debug, Clone)]
pub struct ObservabilityFields {
    /// トレース機能の有効/無効フラグ
    pub trace_enabled: bool,
    /// トレースエンドポイント URL（例: "http://otel-collector.observability:4317"）
    pub trace_endpoint: String,
    /// トレースサンプリングレート（0.0〜1.0）
    pub sample_rate: f64,
    /// ログレベル（例: "info", "debug"）
    pub log_level: String,
    /// ログ出力フォーマット（"text" or "json"）
    pub log_format: String,
}

/// DB プール作成に必要な設定。
///
/// 各サーバーの DatabaseConfig から変換して使用する。
/// サーバー固有の接続 URL 生成ロジック（host/port/name 分割 vs 単一 URL）の
/// 違いを吸収するため、最終的な接続 URL を受け取る設計にしている。
#[cfg(feature = "startup-db")]
#[derive(Debug, Clone)]
pub struct DatabasePoolConfig {
    /// PostgreSQL 接続 URL（postgres://user:pass@host:port/dbname?sslmode=...）
    pub url: String,
    /// 最大接続数
    pub max_connections: u32,
    /// 最小接続数（アイドル時にも維持する接続数）
    pub min_connections: u32,
    /// 接続取得タイムアウト（秒）
    pub connect_timeout_secs: u64,
    /// 接続の最大生存時間（秒、None で無制限）
    pub max_lifetime_secs: Option<u64>,
}

#[cfg(feature = "startup-db")]
impl Default for DatabasePoolConfig {
    /// デフォルト設定を返す。
    /// max_connections=25, min_connections=5, connect_timeout=30秒, max_lifetime=300秒。
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 25,
            min_connections: 5,
            connect_timeout_secs: 30,
            max_lifetime_secs: Some(300),
        }
    }
}

/// JWKS 認証設定（nested 形式: jwt + jwks）。
///
/// 各サーバーの AuthConfig から変換して使用する。
/// サーバー固有の AuthConfig 構造体に依存しないよう分離している。
#[cfg(feature = "startup-auth")]
#[derive(Debug, Clone)]
pub struct JwksAuthConfig {
    /// JWT トークンの検証に使用する issuer / audience 設定
    pub jwt: JwksAuthJwtConfig,
    /// JWKS エンドポイントの設定（オプション）
    pub jwks: Option<JwksAuthJwksConfig>,
}

/// JwksAuthJwtConfig は JWT 検証の issuer / audience を保持する。
#[cfg(feature = "startup-auth")]
#[derive(Debug, Clone)]
pub struct JwksAuthJwtConfig {
    /// JWT 発行者（issuer）
    pub issuer: String,
    /// JWT 対象者（audience）
    pub audience: String,
}

/// JwksAuthJwksConfig は JWKS エンドポイント URL とキャッシュ TTL を保持する。
#[cfg(feature = "startup-auth")]
#[derive(Debug, Clone)]
pub struct JwksAuthJwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュ TTL（秒）
    pub cache_ttl_secs: u64,
}

/// 文字列形式の時間指定（例: "5m", "30s", "1h", "100ms"）を `Duration` に変換する。
///
/// 複数サーバーで重複定義されていた `parse_pool_duration` を server-common に集約し、
/// DB プール設定の `conn_max_lifetime` 解析に使用する。
/// 不正な値・空文字の場合は `None` を返す。
pub fn parse_pool_duration(raw: &str) -> Option<std::time::Duration> {
    if raw.is_empty() {
        return None;
    }
    // 単位別に分岐して Duration に変換する
    if let Some(h) = raw.strip_suffix('h') {
        h.parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 3600))
    } else if let Some(m) = raw.strip_suffix('m') {
        m.parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 60))
    } else if let Some(ms) = raw.strip_suffix("ms") {
        ms.parse::<u64>().ok().map(std::time::Duration::from_millis)
    } else if let Some(s) = raw.strip_suffix('s') {
        s.parse::<u64>().ok().map(std::time::Duration::from_secs)
    } else {
        // 単位なしは秒として解釈する
        raw.parse::<u64>().ok().map(std::time::Duration::from_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ServerBuilder に tier を必須引数として渡せることを確認する（P2-34）。
    #[test]
    fn test_server_builder_with_tier() {
        let builder = ServerBuilder::new("test-server", "0.1.0", "system");
        assert_eq!(builder.service_name(), "test-server");
        assert_eq!(builder.tier, "system");
    }

    /// business tier を指定した ServerBuilder を作成できることを確認する。
    #[test]
    fn test_server_builder_business_tier() {
        let builder = ServerBuilder::new("test-server", "0.1.0", "business");
        assert_eq!(builder.tier, "business");
    }

    /// create_metrics() がサービス名付きの Metrics を返すことを確認する。
    #[test]
    fn test_create_metrics() {
        let builder = ServerBuilder::new("test-server", "0.1.0", "system");
        let metrics = builder.create_metrics();
        // Metrics が生成されていることを確認（内部の service_name は非公開だが生成自体を検証）
        drop(metrics);
    }

    /// DatabasePoolConfig のデフォルト値を確認する。
    #[cfg(feature = "startup-db")]
    #[test]
    fn test_database_pool_config_default() {
        let config = DatabasePoolConfig::default();
        assert_eq!(config.max_connections, 25);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.max_lifetime_secs, Some(300));
    }

    /// ObservabilityFields からテレメトリ初期化を試行する。
    /// 実際のテレメトリ初期化はグローバル状態を変更するためスキップし、
    /// フィールド構築のみ検証する。
    #[test]
    fn test_observability_fields_construction() {
        let fields = ObservabilityFields {
            trace_enabled: true,
            trace_endpoint: "http://localhost:4317".to_string(),
            sample_rate: 1.0,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
        };
        assert!(fields.trace_enabled);
        assert_eq!(fields.sample_rate, 1.0);
    }

    /// JwksAuthConfig のフィールドが正しく設定されることを確認する。
    #[cfg(feature = "startup-auth")]
    #[test]
    fn test_jwks_auth_config_fields() {
        let config = JwksAuthConfig {
            jwt: JwksAuthJwtConfig {
                issuer: "https://auth.example.com".to_string(),
                audience: "k1s0-api".to_string(),
            },
            jwks: Some(JwksAuthJwksConfig {
                url: "https://auth.example.com/.well-known/jwks.json".to_string(),
                cache_ttl_secs: 3600,
            }),
        };
        assert_eq!(config.jwks.as_ref().unwrap().cache_ttl_secs, 3600);
        assert!(config.jwks.as_ref().unwrap().url.contains("jwks.json"));
    }

    /// parse_pool_duration が各単位文字列を正しく Duration に変換することを確認する。
    #[test]
    fn test_parse_pool_duration() {
        assert_eq!(
            parse_pool_duration("5m"),
            Some(std::time::Duration::from_secs(300))
        );
        assert_eq!(
            parse_pool_duration("30s"),
            Some(std::time::Duration::from_secs(30))
        );
        assert_eq!(
            parse_pool_duration("100ms"),
            Some(std::time::Duration::from_millis(100))
        );
        assert_eq!(
            parse_pool_duration("1h"),
            Some(std::time::Duration::from_secs(3600))
        );
        assert_eq!(parse_pool_duration(""), None);
        assert_eq!(parse_pool_duration("invalid"), None);
    }
}
