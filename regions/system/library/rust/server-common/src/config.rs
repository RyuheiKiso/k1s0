//! サーバー設定の共通構造体モジュール。
//!
//! 全サーバーで重複していた ObservabilityConfig / LogConfig / TraceConfig / MetricsConfig を
//! 一箇所に集約し、デフォルト値の変更を1ファイルで完結させる。

use serde::Deserialize;

use crate::DEFAULT_OTEL_ENDPOINT;

// ---------------------------------------------------------------------------
// ObservabilityConfig — ログ・トレース・メトリクスの設定をまとめる親構造体
// ---------------------------------------------------------------------------

/// 可観測性（Observability）設定の親構造体。
/// ログ・トレース・メトリクスそれぞれのサブ設定を保持する。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    /// ログ出力設定
    #[serde(default)]
    pub log: LogConfig,
    /// 分散トレース設定
    #[serde(default)]
    pub trace: TraceConfig,
    /// メトリクス設定
    #[serde(default)]
    pub metrics: MetricsConfig,
}

// ---------------------------------------------------------------------------
// LogConfig — ログレベル・フォーマットの設定
// ---------------------------------------------------------------------------

/// ログ出力に関する設定。
/// level: ログレベル（info, debug, warn, error）
/// format: 出力フォーマット（json, text）
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    /// ログレベル（デフォルト: "info"）
    #[serde(default = "default_log_level")]
    pub level: String,
    /// ログ出力フォーマット（デフォルト: "json"）
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LogConfig {
    /// デフォルト値: level="info", format="json"
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

// ---------------------------------------------------------------------------
// TraceConfig — 分散トレースの設定
// ---------------------------------------------------------------------------

/// 分散トレースに関する設定。
/// OpenTelemetry コレクターへのエクスポート制御を行う。
#[derive(Debug, Clone, Deserialize)]
pub struct TraceConfig {
    /// トレースの有効/無効フラグ（デフォルト: true）
    #[serde(default = "default_trace_enabled")]
    pub enabled: bool,
    /// OpenTelemetry コレクターのエンドポイント URL
    #[serde(default = "default_trace_endpoint")]
    pub endpoint: String,
    /// トレースサンプリングレート（0.0〜1.0、デフォルト: 1.0 = 全リクエスト）
    #[serde(default = "default_trace_sample_rate")]
    pub sample_rate: f64,
}

impl Default for TraceConfig {
    /// デフォルト値: enabled=true, endpoint=DEFAULT_OTEL_ENDPOINT, sample_rate=1.0
    fn default() -> Self {
        Self {
            enabled: default_trace_enabled(),
            endpoint: default_trace_endpoint(),
            sample_rate: default_trace_sample_rate(),
        }
    }
}

// ---------------------------------------------------------------------------
// MetricsConfig — Prometheus メトリクスの設定
// ---------------------------------------------------------------------------

/// メトリクスエンドポイントに関する設定。
/// Prometheus スクレイピング用のパスと有効/無効を制御する。
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    /// メトリクスの有効/無効フラグ（デフォルト: true）
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
    /// メトリクスエンドポイントのパス（デフォルト: "/metrics"）
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

impl Default for MetricsConfig {
    /// デフォルト値: enabled=true, path="/metrics"
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            path: default_metrics_path(),
        }
    }
}

// ---------------------------------------------------------------------------
// デフォルト値関数群 — serde(default = "...") から参照される
// ---------------------------------------------------------------------------

/// ログレベルのデフォルト値を返す。
fn default_log_level() -> String {
    "info".to_string()
}

/// ログフォーマットのデフォルト値を返す。
fn default_log_format() -> String {
    "json".to_string()
}

/// トレース有効フラグのデフォルト値を返す。
fn default_trace_enabled() -> bool {
    true
}

/// トレースエンドポイントのデフォルト値を返す。
/// lib.rs の DEFAULT_OTEL_ENDPOINT 定数を使用し、エンドポイント変更を一箇所に集約する。
fn default_trace_endpoint() -> String {
    DEFAULT_OTEL_ENDPOINT.to_string()
}

/// トレースサンプリングレートのデフォルト値を返す。
fn default_trace_sample_rate() -> f64 {
    1.0
}

/// メトリクス有効フラグのデフォルト値を返す。
fn default_metrics_enabled() -> bool {
    true
}

/// メトリクスエンドポイントパスのデフォルト値を返す。
fn default_metrics_path() -> String {
    "/metrics".to_string()
}

// ---------------------------------------------------------------------------
// ObservabilityFields への変換 — startup モジュールとの連携
// ---------------------------------------------------------------------------

/// ObservabilityConfig から startup::ObservabilityFields への変換。
/// サーバーの startup.rs で手動変換していたコードを不要にする。
#[cfg(feature = "startup")]
impl From<&ObservabilityConfig> for crate::startup::ObservabilityFields {
    fn from(config: &ObservabilityConfig) -> Self {
        Self {
            trace_enabled: config.trace.enabled,
            trace_endpoint: config.trace.endpoint.clone(),
            sample_rate: config.trace.sample_rate,
            log_level: config.log.level.clone(),
            log_format: config.log.format.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// ObservabilityConfig のデフォルト値が期待通りであることを確認する。
    #[test]
    fn test_observability_config_defaults() {
        let cfg = ObservabilityConfig::default();
        assert_eq!(cfg.log.level, "info");
        assert_eq!(cfg.log.format, "json");
        assert!(cfg.trace.enabled);
        assert_eq!(cfg.trace.endpoint, DEFAULT_OTEL_ENDPOINT);
        assert!((cfg.trace.sample_rate - 1.0).abs() < f64::EPSILON);
        assert!(cfg.metrics.enabled);
        assert_eq!(cfg.metrics.path, "/metrics");
    }

    /// YAML からの逆シリアル化でデフォルト値が適用されることを確認する。
    #[test]
    fn test_observability_config_deserialization_defaults() {
        let yaml = "{}";
        let cfg: ObservabilityConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.log.level, "info");
        assert!(cfg.trace.enabled);
        assert!(cfg.metrics.enabled);
    }

    /// YAML で明示的に値を指定した場合にデフォルト値が上書きされることを確認する。
    #[test]
    fn test_observability_config_deserialization_overrides() {
        let yaml = r#"
log:
  level: "debug"
  format: "text"
trace:
  enabled: false
  endpoint: "http://custom:4317"
  sample_rate: 0.5
metrics:
  enabled: false
  path: "/custom-metrics"
"#;
        let cfg: ObservabilityConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.log.level, "debug");
        assert_eq!(cfg.log.format, "text");
        assert!(!cfg.trace.enabled);
        assert_eq!(cfg.trace.endpoint, "http://custom:4317");
        assert!((cfg.trace.sample_rate - 0.5).abs() < f64::EPSILON);
        assert!(!cfg.metrics.enabled);
        assert_eq!(cfg.metrics.path, "/custom-metrics");
    }

    /// LogConfig のデフォルト値を確認する。
    #[test]
    fn test_log_config_default() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, "json");
    }

    /// TraceConfig のデフォルト値を確認する。
    #[test]
    fn test_trace_config_default() {
        let cfg = TraceConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.endpoint, DEFAULT_OTEL_ENDPOINT);
        assert!((cfg.sample_rate - 1.0).abs() < f64::EPSILON);
    }

    /// MetricsConfig のデフォルト値を確認する。
    #[test]
    fn test_metrics_config_default() {
        let cfg = MetricsConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.path, "/metrics");
    }
}
