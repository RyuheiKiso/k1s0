// error.rs — k1s0 tier1 Library core: 共通エラー型
// tier1 Library 全モジュールで使用する統一エラー型を定義する。
// 公開 API シグネチャでは anyhow::Error ではなく LibraryError を使う。
// OSS 固有のエラー型（sqlx::Error 等）はこの型に変換して外部に露出しない。

// thiserror は使わず anyhow + 独自 enum を組み合わせて依存を最小化する
use std::fmt;

// LibraryError は tier1 Library が返す最上位エラー型。
// 全カテゴリのエラーを domain 語彙で分類する（OSS エラー文字列は message に隠蔽する）。
#[derive(Debug)]
pub enum LibraryError {
    // AuthError: 認証・認可に関するエラー（token 無効 / scope 不足 / step_up 未達等）
    AuthError {
        // message: エラーの内容を日本語で記述する（OSS スタック文字列は含まない）
        message: String,
    },
    // SecretError: 鍵管理・シークレット取得に関するエラー（OpenBao 通信失敗等）
    SecretError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // CacheError: KeyValue / Cache 操作に関するエラー（TTL 計算失敗含む）
    CacheError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // StorageError: Object Storage 操作に関するエラー（bucket 不正 / upload 失敗等）
    StorageError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // RpcError: RPC / Gateway に関するエラー（timeout / circuit open / retry 枯渇等）
    RpcError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // MessagingError: Messaging / EventBus に関するエラー（Kafka 送受信失敗等）
    MessagingError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // SchemaError: Schema Registry に関するエラー（スキーマ未登録 / 互換性違反等）
    SchemaError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // DbError: リレーショナル DB に関するエラー（接続失敗 / constraint 違反等）
    DbError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // VectorError: Vector Search に関するエラー（インデックス構築失敗等）
    VectorError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // WorkflowError: Workflow / Long-running Saga に関するエラー（step 失敗 / timeout 等）
    WorkflowError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // RulesError: Rule Engine に関するエラー（ルール評価失敗等）
    RulesError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // ConfigError: Configuration / Feature Flag に関するエラー（設定取得失敗等）
    ConfigError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // ObservabilityError: Observability（logging / tracing / metrics / profiling）に関するエラー
    ObservabilityError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // PolicyError: retry / timeout / circuit breaker ポリシー違反に関するエラー
    PolicyError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
    // InternalError: tier1 Library 内部不変条件違反（バグ相当）
    InternalError {
        // message: エラーの内容を日本語で記述する
        message: String,
    },
}

// LibraryError の Display 実装: domain 語彙でエラーを表示する
impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 各バリアントのメッセージを書き出す
        match self {
            // AuthError の表示
            LibraryError::AuthError { message } => write!(f, "AuthError: {message}"),
            // SecretError の表示
            LibraryError::SecretError { message } => write!(f, "SecretError: {message}"),
            // CacheError の表示
            LibraryError::CacheError { message } => write!(f, "CacheError: {message}"),
            // StorageError の表示
            LibraryError::StorageError { message } => write!(f, "StorageError: {message}"),
            // RpcError の表示
            LibraryError::RpcError { message } => write!(f, "RpcError: {message}"),
            // MessagingError の表示
            LibraryError::MessagingError { message } => write!(f, "MessagingError: {message}"),
            // SchemaError の表示
            LibraryError::SchemaError { message } => write!(f, "SchemaError: {message}"),
            // DbError の表示
            LibraryError::DbError { message } => write!(f, "DbError: {message}"),
            // VectorError の表示
            LibraryError::VectorError { message } => write!(f, "VectorError: {message}"),
            // WorkflowError の表示
            LibraryError::WorkflowError { message } => write!(f, "WorkflowError: {message}"),
            // RulesError の表示
            LibraryError::RulesError { message } => write!(f, "RulesError: {message}"),
            // ConfigError の表示
            LibraryError::ConfigError { message } => write!(f, "ConfigError: {message}"),
            // ObservabilityError の表示
            LibraryError::ObservabilityError { message } => {
                // ObservabilityError のメッセージを書き出す
                write!(f, "ObservabilityError: {message}")
            }
            // PolicyError の表示
            LibraryError::PolicyError { message } => write!(f, "PolicyError: {message}"),
            // InternalError の表示
            LibraryError::InternalError { message } => write!(f, "InternalError: {message}"),
        }
    }
}

// std::error::Error を実装して anyhow と組み合わせられるようにする
impl std::error::Error for LibraryError {}
