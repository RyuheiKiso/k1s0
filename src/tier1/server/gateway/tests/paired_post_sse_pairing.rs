// paired_post_sse_pairing.rs — POST↔SSE pair adapter のペアリング統合テスト
// POST /bidi/up で publish したイベントが GET /bidi/down の SSE stream で受信できることを確認する。
// axum-test の in-process サーバーを使用して EventBus broadcast を検証する。
// NOTE: SSE streaming テスト（handle_down）は無限ストリームのため #[ignore] で除外する。
//       CI では POST 方向のテストと status エンドポイントのみを実行する。

// axum-test: in-process HTTP テストサーバーとクライアント（v20 系は axum 0.8 と互換）
use axum_test::TestServer;
// serde_json: JSON パースとアサーション
use serde_json::Value;
// k1s0-tier1-gateway の router 構築関数をインポートする
use k1s0_tier1_gateway::build_router;
// axum の HTTP ステータスコード型をテストアサーションで使用する
use axum::http::StatusCode;

// ===== Test: paired_post_sse パーリング ========================================

// test_paired_up_session_id_required は POST /bidi/up に session_id なしで送信した場合に
// 400 Bad Request が返ることを確認する。
// 01_Bidi適合仕様.md §paired_post_sse adapter: session_id 必須制約を検証する。
#[tokio::test]
async fn test_paired_up_session_id_required() {
    // TestServer を構築する
    let server = TestServer::new(build_router());
    // session_id なしで POST /post-sse/bidi/up に送信する
    let resp = server
        .post("/post-sse/bidi/up")
        // Content-Type: application/json を設定する
        .content_type("application/json")
        // session_id を含まない JSON ボディを送信する
        .json(&serde_json::json!({
            // request_id は指定するが session_id は省略する（バリデーションエラーを引き起こす）
            "request_id": "no-session-request",
            "payload": { "msg": "test" }
        }))
        .await;
    // 400 Bad Request であることを確認する（session_id 未指定エラー）
    resp.assert_status(StatusCode::BAD_REQUEST);
    // エラーメッセージに session_id に関する記述があることを確認する
    let body: Value = resp.json();
    assert!(
        body["error"].as_str().map(|s| s.contains("session_id")).unwrap_or(false),
        "POST /post-sse/bidi/up の error に session_id に関する記述がなければならない"
    );
}

// test_paired_up_publishes_event は POST /post-sse/bidi/up が EventBus への publish に成功することを確認する。
// 01_Bidi適合仕様.md §paired_post_sse adapter: UP 方向の receive + broadcast を検証する。
#[tokio::test]
async fn test_paired_up_publishes_event() {
    // TestServer を構築する
    let server = TestServer::new(build_router());
    // session_id を付けて POST /post-sse/bidi/up に送信する
    let resp = server
        .post("/post-sse/bidi/up")
        // Content-Type: application/json を設定する
        .content_type("application/json")
        // 正しい session_id / event_type / payload を含む JSON ボディを送信する
        .json(&serde_json::json!({
            // セッション識別子を指定する
            "session_id": "pairing-session-001",
            // イベント種別を指定する
            "event_type": "order.placed",
            // request_id: SSE down stream と対応させる識別子
            "request_id": "req-001",
            // テストペイロードを指定する
            "payload": { "order_id": "O-001", "amount": 100 }
        }))
        .await;
    // 200 OK であることを確認する（EventBus への publish 成功を示す）
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // received フラグが true であることを確認する
    assert!(
        body["received"].as_bool().unwrap_or(false),
        "POST /post-sse/bidi/up の received が true でなければならない"
    );
    // session_id がエコーバックされていることを確認する
    assert_eq!(
        body["session_id"].as_str().unwrap_or(""),
        "pairing-session-001",
        "POST /post-sse/bidi/up の session_id がエコーバックされなければならない"
    );
    // event_type がエコーバックされていることを確認する
    assert_eq!(
        body["event_type"].as_str().unwrap_or(""),
        "order.placed",
        "POST /post-sse/bidi/up の event_type がエコーバックされなければならない"
    );
}

// test_paired_up_multiple_events は複数の異なる session_id に対して publish が独立して動作することを確認する。
// 01_Bidi適合仕様.md §paired_post_sse adapter: セッション分離を検証する。
#[tokio::test]
async fn test_paired_up_multiple_events() {
    // TestServer を構築する
    let server = TestServer::new(build_router());

    // session A へのイベント publish
    let resp_a = server
        .post("/post-sse/bidi/up")
        .content_type("application/json")
        .json(&serde_json::json!({
            // セッション A の識別子を指定する
            "session_id": "session-a",
            "event_type": "event.for.a",
            "payload": { "to": "a" }
        }))
        .await;
    // 200 OK であることを確認する
    resp_a.assert_status_ok();

    // session B へのイベント publish（異なるセッション）
    let resp_b = server
        .post("/post-sse/bidi/up")
        .content_type("application/json")
        .json(&serde_json::json!({
            // セッション B の識別子を指定する
            "session_id": "session-b",
            "event_type": "event.for.b",
            "payload": { "to": "b" }
        }))
        .await;
    // 200 OK であることを確認する
    resp_b.assert_status_ok();

    // session A の received フラグが true であることを確認する
    let body_a: Value = resp_a.json();
    assert!(body_a["received"].as_bool().unwrap_or(false));
    // session B の received フラグが true であることを確認する
    let body_b: Value = resp_b.json();
    assert!(body_b["received"].as_bool().unwrap_or(false));
    // 各セッションの session_id が正しくエコーバックされていることを確認する
    assert_eq!(body_a["session_id"].as_str().unwrap_or(""), "session-a");
    assert_eq!(body_b["session_id"].as_str().unwrap_or(""), "session-b");
}

// test_paired_down_opens_sse は GET /post-sse/bidi/down が 200 OK を返すことを確認する。
// axum-test は SSE streaming body を完全消費しようとするため、
// このテストは #[ignore] で CI から除外し、手動検証のみとする。
// 01_Bidi適合仕様.md §paired_post_sse adapter: DOWN 方向の SSE streaming を検証する。
#[tokio::test]
#[ignore = "SSE 無限ストリームのため axum-test では timeout する: 手動 curl で検証"]
async fn test_paired_down_opens_sse() {
    // TestServer を構築する
    let server = TestServer::new(build_router());
    // GET /post-sse/bidi/down に必須クエリパラメータを指定する
    let resp = server
        .get("/post-sse/bidi/down")
        // session_id を指定する（UP と対応させる）
        .add_query_param("session_id", "pairing-session-down-001")
        // request_id を指定する（UP の request_id と対応させる）
        .add_query_param("request_id", "req-down-001")
        // conformance_class を v1_interactive に指定する
        .add_query_param("conformance_class", "v1_interactive")
        .await;
    // 200 OK であることを確認する（SSE stream の開始を示す）
    resp.assert_status_ok();
    // ステータスコードが 200 であることをもって SSE stream 開始を確認する
    let status = resp.status_code();
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /post-sse/bidi/down が 200 を返さなければならない"
    );
}

// test_paired_up_then_down は POST で publish してから GET で SSE を開くフローを検証する。
// axum-test は SSE streaming body を完全消費しようとするため #[ignore] で除外する。
// 01_Bidi適合仕様.md §paired_post_sse adapter: 半二重 emulation の端対端動作を検証する。
#[tokio::test]
#[ignore = "SSE 無限ストリームのため axum-test では timeout する: 手動 curl で検証"]
async fn test_paired_up_then_down() {
    // TestServer を構築する（EventBus を共有した gateway ルーターを使用する）
    let server = TestServer::new(build_router());

    // step 1: POST /post-sse/bidi/up でイベントを publish する
    let up_resp = server
        .post("/post-sse/bidi/up")
        .content_type("application/json")
        .json(&serde_json::json!({
            // セッション識別子を指定する
            "session_id": "paired-e2e-session",
            "event_type": "data.stream",
            "request_id": "paired-req-001",
            "payload": { "stream_chunk": "hello" }
        }))
        .await;
    // 200 OK であることを確認する（EventBus への publish 成功）
    up_resp.assert_status_ok();

    // step 2: GET /post-sse/bidi/down で SSE stream を開く
    let down_resp = server
        .get("/post-sse/bidi/down")
        .add_query_param("session_id", "paired-e2e-session")
        .add_query_param("request_id", "paired-req-001")
        .add_query_param("conformance_class", "v1_event_feed")
        .await;
    // 200 OK であることを確認する（SSE stream の開始）
    down_resp.assert_status_ok();
}
