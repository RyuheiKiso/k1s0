// main.rs — k1s0 tier1 gateway: HTTP/2 (h2c) サーバーエントリポイント
// spec 01 Bidi: 5 conformance_class × 8 adapter の transport 基盤
// spec 03 観測: OTel tracing + W3C traceparent 伝搬
// spec 05 鍵管理: KeyHandle opaque 型による生 key bytes 隠蔽
//
// TLS / ALPN h2 強制は Envoy proxy が担当する（mTLS サービスメッシュ構成）。
// gateway は h2c（HTTP/2 cleartext）で listen し、Envoy の後段として動作する。
// Envoy の ALPN ネゴシエーションで h2 のみを許可する設定は src/_crosscutting/01_http2_enforcement/ を参照。
//
// ルーター実装は lib.rs の build_router() に集約されている。
// main.rs は起動処理（tracing 初期化・listen・serve）のみを担当する。

// tracing-subscriber: サブスクライバー初期化
use tracing_subscriber::EnvFilter;
// tracing: 構造化ロギング
use tracing::info;
// 標準ライブラリ
use std::env;

// アプリケーションエントリポイント
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing を初期化する（RUST_LOG 環境変数で log level を制御する）
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .init();
    // リスニングアドレスを環境変数から取得する（デフォルト: 0.0.0.0:8080）
    let addr = env::var("GATEWAY_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    // TCP リスナーを起動する（h2c モード: Envoy が TLS + ALPN 処理を担当する）
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(
        addr = %addr,
        mode = "h2c-behind-envoy",
        "k1s0-tier1-gateway starting"
    );
    // lib.rs の build_router() を呼び出して axum serve する
    axum::serve(listener, k1s0_tier1_gateway::build_router()).await?;
    Ok(())
}
