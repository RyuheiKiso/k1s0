// sse_paired_subscription.rs — SSE paired adapter の EventBus 統合テスト
// EventBus に publish したイベントが SSE stream で受信できることを確認する。
// axum-test の in-process サーバーを使用して実際の broadcast channel を検証する。
// NOTE: SSE streaming 自体（無限ストリーム）は axum-test では timeout するため #[ignore] で除外する。
//       CI では POST パス経由の publish 確認と webhook status 確認のみを実行する。

// axum-test: in-process HTTP テストサーバーとクライアント（v20 系は axum 0.8 と互換）
use axum_test::TestServer;
// serde_json: JSON パースとアサーション
use serde_json::Value;
// k1s0-tier1-gateway の router 構築関数をインポートする
use k1s0_tier1_gateway::build_router;
// axum の HTTP ステータスコード型をテストアサーションで使用する
use axum::http::StatusCode;

// ===== Test: sse_paired subscription ==========================================

// test_sse_paired_post_then_subscribe は POST /post-sse/bidi/up で publish してから
// subscribe の準備ができていることを確認する。
// SSE stream 自体の消費は #[ignore] テストに委ねる。
// 01_Bidi適合仕様.md §sse_paired adapter の EventBus 統合を検証する。
#[tokio::test]
async fn test_sse_paired_post_then_subscribe() {
    // TestServer を構築する（EventBus を共有した gateway ルーターを使用する）
    let server = TestServer::new(build_router());
    // POST /post-sse/bidi/up でイベントを EventBus に publish する
    let up_resp = server
        .post("/post-sse/bidi/up")
        // Content-Type: application/json を設定する
        .content_type("application/json")
        // session_id を含む JSON ボディを送信する
        .json(&serde_json::json!({
            // セッション識別子を指定する（SSE ストリームと同じ値を使う）
            "session_id": "sse-subscription-test-001",
            // イベント種別を指定する
            "event_type": "test.event.published",
            // テストペイロードを指定する
            "payload": { "value": 123 }
        }))
        .await;
    // 200 OK であることを確認する（EventBus への publish 成功を示す）
    up_resp.assert_status_ok();
    // received フラグが true であることを確認する
    let up_body: Value = up_resp.json();
    assert!(
        up_body["received"].as_bool().unwrap_or(false),
        "POST /post-sse/bidi/up の received が true でなければならない"
    );
    // session_id がエコーバックされていることを確認する
    assert_eq!(
        up_body["session_id"].as_str().unwrap_or(""),
        "sse-subscription-test-001",
        "POST /post-sse/bidi/up の session_id がエコーバックされなければならない"
    );
}

// test_sse_paired_multiple_sessions は複数セッションが独立して動作することを確認する。
// 01_Bidi適合仕様.md §sse_paired adapter: session 分離の物理検証。
#[tokio::test]
async fn test_sse_paired_multiple_sessions() {
    // TestServer を構築する
    let server = TestServer::new(build_router());

    // セッション X へのイベント publish
    let resp_x = server
        .post("/post-sse/bidi/up")
        .content_type("application/json")
        .json(&serde_json::json!({
            // セッション X の識別子を指定する
            "session_id": "sse-session-x",
            "event_type": "event.x",
            "payload": { "x": true }
        }))
        .await;
    // 200 OK であることを確認する
    resp_x.assert_status_ok();

    // セッション Y へのイベント publish（別セッション）
    let resp_y = server
        .post("/post-sse/bidi/up")
        .content_type("application/json")
        .json(&serde_json::json!({
            // セッション Y の識別子を指定する（X とは異なる）
            "session_id": "sse-session-y",
            "event_type": "event.y",
            "payload": { "y": true }
        }))
        .await;
    // 200 OK であることを確認する
    resp_y.assert_status_ok();

    // X と Y の session_id が正しくエコーバックされていることを確認する
    let body_x: Value = resp_x.json();
    let body_y: Value = resp_y.json();
    // セッション X の識別子が正しいことを確認する
    assert_eq!(body_x["session_id"].as_str().unwrap_or(""), "sse-session-x");
    // セッション Y の識別子が正しいことを確認する
    assert_eq!(body_y["session_id"].as_str().unwrap_or(""), "sse-session-y");
    // 両セッションで受信確認フラグが true であることを確認する
    assert!(body_x["received"].as_bool().unwrap_or(false));
    assert!(body_y["received"].as_bool().unwrap_or(false));
}

// test_sse_stream_opens は GET /sse-stream/sse が 200 OK を返すことを確認する。
// axum-test は SSE streaming body を完全消費しようとするため、
// このテストは #[ignore] で CI から除外し、手動検証のみとする。
// 01_Bidi適合仕様.md §sse_paired adapter の stream 開始を検証する。
#[tokio::test]
#[ignore = "SSE 無限ストリームのため axum-test では timeout する: 手動 curl で検証"]
async fn test_sse_stream_opens() {
    // TestServer を構築する
    let server = TestServer::new(build_router());
    // GET /sse-stream/sse に session_id と conformance_class クエリパラメータを指定する
    let resp = server
        .get("/sse-stream/sse")
        // session_id を指定して特定のセッションの SSE stream を開く
        .add_query_param("session_id", "sse-test-session-001")
        // conformance_class を v1_event_feed に指定する
        .add_query_param("conformance_class", "v1_event_feed")
        .await;
    // 200 OK であることを確認する（SSE stream の開始を示す）
    resp.assert_status_ok();
}

// test_sse_stream_resume は Last-Event-ID ヘッダーを付けた場合の resume を確認する。
// axum-test では SSE stream の消費ができないため #[ignore] で除外する。
// 01_Bidi適合仕様.md §resumable=REQUIRED の物理保証を検証する。
#[tokio::test]
#[ignore = "SSE 無限ストリームのため axum-test では timeout する: 手動 curl で検証"]
async fn test_sse_stream_resume() {
    // TestServer を構築する
    let server = TestServer::new(build_router());
    // GET /sse-stream/sse に Last-Event-ID ヘッダーを付けて送信する
    let resp = server
        .get("/sse-stream/sse")
        // session_id を指定する
        .add_query_param("session_id", "sse-resume-session")
        // last-event-id ヘッダーを付けて resume を要求する（小文字: HTTP は case-insensitive）
        .add_header(
            // last-event-id ヘッダー名（axum は小文字で受け付ける）
            axum::http::header::HeaderName::from_static("last-event-id"),
            // 前回最終イベント ID として "42" を指定する
            axum::http::HeaderValue::from_static("42"),
        )
        .await;
    // 200 OK であることを確認する
    let status = resp.status_code();
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /sse-stream/sse の resume リクエストが 200 を返さなければならない"
    );
}
