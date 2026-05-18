// mod.rs — k1s0 tier1 Library core: 横断的・基盤モジュール群
// auth / observability / profiling / secret / error / policy の 6 サブモジュールを提供する。
// both（frontend / backend 共通）または backend 専用の注記を各モジュールに記載する。
// 公開 API の型はこのモジュール経由で参照できるように re-export する。

// auth モジュール: Authentication/Authorization L3（both 向け）
// AuthClass / AuthContext / AuthVerifier / ScopeRequirement 等を提供する
pub mod auth;

// observability モジュール: Logging / Tracing / Metrics L3（both 向け）
// Logger / Tracer / Meter / LogRecord / SpanContext / MetricPoint 等を提供する
pub mod observability;

// profiling モジュール: Profiling L2*（backend 専用）
// Profiler / ProfileRequest / ProfileData / ProfileKind 等を提供する
pub mod profiling;

// secret モジュール: Secret Management L3（backend 専用）
// KeyClass / KeyHandle / OpenBaoKeyHandle / SecretStore / SecretMetadata 等を提供する
pub mod secret;

// error モジュール: 共通エラー型（both 向け）
// LibraryError を提供する（全カテゴリのエラーを domain 語彙で分類する）
pub mod error;

// policy モジュール: retry / timeout / circuit breaker ポリシー（both 向け）
// ServicePolicy / RetryPolicy / TimeoutPolicy / CircuitBreakerPolicy 等を提供する
pub mod policy;

// 頻繁に使用する型を core 名前空間から直接参照できるように re-export する
// auth 関連: 最頻出の型を re-export する
pub use auth::{AuthClass, AuthContext, AuthVerificationResult, AuthVerifier, ScopeRequirement};
// error 関連: LibraryError を直接参照できるようにする
pub use error::LibraryError;
// policy 関連: ServicePolicy を直接参照できるようにする
pub use policy::{CircuitBreakerPolicy, CircuitBreakerState, RetryPolicy, ServicePolicy, TimeoutPolicy};
// observability 関連: 主要型を re-export する
pub use observability::{
    // ロギング関連
    LogLevel,
    LogRecord,
    Logger,
    // トレーシング関連
    SpanContext,
    SpanStatus,
    Tracer,
    // メトリクス関連
    Meter,
    MetricKind,
    MetricPoint,
};
// secret 関連: 主要型を re-export する
pub use secret::{KeyClass, KeyHandle, OpenBaoKeyHandle, SecretMetadata, SecretStore};
// profiling 関連: 主要型を re-export する
pub use profiling::{ProfileData, ProfileKind, ProfileRequest, Profiler};
