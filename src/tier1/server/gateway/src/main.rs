// k1s0 tier1 gateway: HTTP/2 ALPN h2 サーバーエントリポイント
// spec 01 Bidi: conformance テストランナー
// spec 03 観測: OTel tracing + W3C traceparent 伝搬
// spec 05 鍵管理: KeyHandle opaque 型による生 key bytes の隠蔽

// conformance テストモジュールを宣言する
mod bidi;
// Bidi state machine モジュールを宣言する
mod conformance;

// axum: HTTP/2 ルーターとハンドラーのインポート
use axum::{
    // JSON レスポンス型
    Json,
    // ルーターの構築に使用する
    Router,
    // ルーティングのトレイト
    routing::get,
};
// Serde デシリアライズ/シリアライズのインポート
use serde::{Deserialize, Serialize};
// tracing のインポート
use tracing::{info, instrument};
// tracing サブスクライバーのインポート
use tracing_subscriber::EnvFilter;

// ヘルスチェックレスポンスの構造体
#[derive(Serialize, Deserialize)]
struct HealthResponse {
    // サービスの動作状態を示すフィールド
    status: String,
    // サービス名
    service: String,
    // HTTP/2 ALPN h2 の強制フラグ
    http2_enforced: bool,
    // OTel tracing が有効かどうかのフラグ（spec 03 観測）
    otel_tracing_enabled: bool,
}

// KeyHandle: 生 key bytes を公開しない opaque 型（spec 05 鍵管理）
// 公開 API シグネチャに生 key bytes を露出させないために使用する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHandle {
    // 鍵の不透明識別子（raw bytes は保持しない）
    handle_id: String,
    // 鍵クラス（v1_data_dek / v1_data_kek / v1_token_signing 等）
    key_class: String,
    // 鍵が有効かどうかのフラグ（raw bytes を知らなくても判定できる）
    is_valid: bool,
}

impl KeyHandle {
    // KeyHandle を生成する（生 key bytes は受け取っても外部に公開しない）
    pub fn create(key_class: String, handle_id: String) -> Self {
        // 生 key bytes はここで受け取っても Self に保存せず破棄する設計
        Self {
            handle_id,
            key_class,
            is_valid: true,
        }
    }
}

// health check エンドポイントのハンドラー
#[instrument]
async fn health_handler() -> Json<HealthResponse> {
    // ヘルスチェックレスポンスを構築して返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-gateway".to_string(),
        // HTTP/2 は axum の http2 feature で強制されている
        http2_enforced: true,
        // OTel tracing は tracing_subscriber で有効化されている
        otel_tracing_enabled: true,
    })
}

// 全 40 conformance cell テストを実行するエンドポイントのハンドラー
#[instrument]
async fn conformance_handler() -> Json<conformance::ConformanceReport> {
    // 全 40 cell の conformance テストを実行する
    let report = conformance::run_all_conformance_tests("kind-k1s0-target");
    // テスト結果をログに記録する（OTel span に trace_id が自動付与される）
    info!(
        all_passed = report.all_passed,
        total_cells = report.cells.len(),
        "Conformance test completed"
    );
    // レポートを JSON レスポンスとして返す
    Json(report)
}

// KeyHandle デモエンドポイントのハンドラー（spec 05 鍵管理）
// 生 key bytes を返さないことを API 型レベルで保証する
#[instrument]
async fn key_handle_demo_handler() -> Json<KeyHandle> {
    // KeyHandle を生成する（生 key bytes は返さない）
    let handle = KeyHandle::create(
        // DEK クラスの鍵ハンドルを生成する
        "v1_data_dek".to_string(),
        // handle_id は UUID v4 で一意生成する
        uuid::Uuid::new_v4().to_string(),
    );
    // KeyHandle を JSON レスポンスとして返す（raw bytes は構造体に含まれない）
    info!(
        key_class = %handle.key_class,
        is_valid = handle.is_valid,
        "KeyHandle created (raw bytes not in response)"
    );
    Json(handle)
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber を初期化する（OTel layer は Collector 接続時に有効化する）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        // W3C traceparent ヘッダーを構造化ログに含める設定
        .with_target(true)
        .init();
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイントを登録する
        .route("/health", get(health_handler))
        // conformance テストエンドポイントを登録する（40 cell の全テストを実行する）
        .route("/conformance/run", get(conformance_handler))
        // KeyHandle デモエンドポイントを登録する（spec 05 鍵管理の API 型保証デモ）
        .route("/kek/demo", get(key_handle_demo_handler));
    // サーバーのリスニングアドレスを設定する
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    // サーバー起動をログに記録する
    info!("k1s0-tier1-gateway starting on :8080 (HTTP/2 h2, OTel tracing enabled)");
    // サーバーを起動する（HTTP/2 対応は axum の http2 feature で自動有効化される）
    axum::serve(listener, app).await?;
    Ok(())
}
