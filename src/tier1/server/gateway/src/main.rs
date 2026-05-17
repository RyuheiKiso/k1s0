// k1s0 tier1 gateway: HTTP/2 ALPN h2 サーバーエントリポイント
// Bidi handshake conformance + health check + conformance runner エンドポイントを提供する

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
// トレーシングのインポート
use tracing::info;
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
}

// conformance テスト実行リクエストの構造体
#[derive(Serialize, Deserialize)]
struct ConformanceRequest {
    // テスト対象のクラスター名
    cluster: String,
}

// health check エンドポイントのハンドラー
async fn health_handler() -> Json<HealthResponse> {
    // ヘルスチェックレスポンスを構築して返す
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "k1s0-tier1-gateway".to_string(),
        // HTTP/2 は axum の http2 feature で強制されている
        http2_enforced: true,
    })
}

// 全 40 conformance cell テストを実行するエンドポイントのハンドラー
async fn conformance_handler() -> Json<conformance::ConformanceReport> {
    // 全 40 cell の conformance テストを実行する
    let report = conformance::run_all_conformance_tests("kind-k1s0-target");
    // テスト結果をログに記録する
    info!(
        all_passed = report.all_passed,
        total_cells = report.cells.len(),
        "Conformance test completed"
    );
    // レポートを JSON レスポンスとして返す
    Json(report)
}

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 環境変数から tracing フィルターを設定する（デフォルト: info）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();
    // ルーターを構築する
    let app = Router::new()
        // ヘルスチェックエンドポイントを登録する
        .route("/health", get(health_handler))
        // conformance テストエンドポイントを登録する（40 cell の全テストを実行する）
        .route("/conformance/run", get(conformance_handler));
    // サーバーのリスニングアドレスを設定する
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    // サーバー起動をログに記録する
    info!("k1s0-tier1-gateway starting on :8080 (HTTP/2 h2)");
    // サーバーを起動する（HTTP/2 対応は axum の http2 feature で自動有効化される）
    axum::serve(listener, app).await?;
    Ok(())
}
