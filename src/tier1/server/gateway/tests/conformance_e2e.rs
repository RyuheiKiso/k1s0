// conformance_e2e.rs — tier1 gateway integration test
// axum-test を使って in-process で各 adapter endpoint を叩き、Bidi conformance を検証する。
// 01_Bidi適合仕様.md §adapter↔class supports 対応 の全エンドポイントをカバーする。
// Docker / Testcontainers は不要（axum-test が in-process HTTP サーバーを起動する）。

// axum-test: in-process HTTP テストサーバーとクライアント（v20 系は axum 0.8 と互換）
use axum_test::TestServer;
// serde_json: JSON パースとアサーション
use serde_json::Value;
// axum の HTTP ステータスコード型をテストアサーションで使用する
use axum::http::StatusCode;
// bytes: .bytes() API に渡す Bytes 型をインポートする
use bytes::Bytes;
// k1s0-tier1-gateway の router 構築関数をインポートする
use k1s0_tier1_gateway::build_router;

// ===== ヘルパー関数 ============================================================

// new_server はテスト用 axum-test TestServer を構築して返す。
// build_router() で gateway の全 adapter エンドポイントを登録する。
// v20 では TestServer::new() は Result ではなく Self を返す。
fn new_server() -> TestServer {
    // gateway ルーターを取得する
    let app = build_router();
    // axum-test TestServer を起動する（v20 の new() は infallible）
    TestServer::new(app)
}

// ===== Test 1: /health =========================================================

// test_health_endpoint は GET /health が 200 OK と { "status": "healthy" } を返すことを検証する。
// 01_Bidi適合仕様.md §liveness probe 要件を満たすかを確認する。
#[tokio::test]
async fn test_health_endpoint() {
    // TestServer を構築する
    let server = new_server();
    // GET /health を送信する
    let resp = server.get("/health").await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // status フィールドが "healthy" であることを確認する
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "healthy",
        "/health の status フィールドが 'healthy' でなければならない"
    );
    // service フィールドが存在することを確認する
    assert!(
        body["service"].is_string(),
        "/health の service フィールドが文字列でなければならない"
    );
}

// ===== Test 2: /conformance/run ================================================

// test_conformance_run は GET /conformance/run が 200 OK と
// { "all_passed": bool, "cells": [...] } を返すことを検証する。
// 01_Bidi適合仕様.md §Bidi conformance テストスイートの完全性を確認する。
#[tokio::test]
async fn test_conformance_run() {
    // TestServer を構築する
    let server = new_server();
    // GET /conformance/run を送信する
    let resp = server.get("/conformance/run").await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // all_passed フィールドが bool であることを確認する
    assert!(
        body["all_passed"].is_boolean(),
        "/conformance/run の all_passed フィールドが bool でなければならない"
    );
    // cells フィールドが配列であることを確認する
    assert!(
        body["cells"].is_array(),
        "/conformance/run の cells フィールドが配列でなければならない"
    );
    // cells が 40 要素（8 adapter × 5 conformance_class）であることを確認する
    let cells = body["cells"].as_array().expect("cells が配列でなければならない");
    assert_eq!(
        cells.len(),
        40,
        "/conformance/run の cells は 40 要素（5 class × 8 adapter）でなければならない"
    );
    // 全セルが green であることを確認する（in-process stub は必ず green）
    for cell in cells {
        // 各セルの status フィールドを取得する
        let status = cell["status"].as_str().unwrap_or("unknown");
        // セルの cell_id を取得してエラーメッセージに含める
        let cell_id = cell["cell_id"].as_str().unwrap_or("unknown_id");
        // status が green であることを確認する
        assert_eq!(
            status, "green",
            "conformance cell {} の status が green でなければならない",
            cell_id
        );
    }
}

// ===== Test 3: /grpc/... — Content-Type: application/grpc 以外を拒否する =======

// test_grpc_native_content_type_reject は POST /grpc/... に
// Content-Type: application/grpc なしで送信した場合に 415 を返すことを検証する。
// 01_Bidi適合仕様.md §grpc_native adapter の Content-Type 強制を確認する。
#[tokio::test]
async fn test_grpc_native_content_type_reject() {
    // TestServer を構築する
    let server = new_server();
    // Content-Type: text/plain で gRPC エンドポイントに POST する
    // .bytes() は Bytes 型を受け取る（v20 API）
    let resp = server
        .post("/grpc/k1s0.tier1.bidi.v1.BidiService/OpenBidiStream")
        // Content-Type を application/grpc 以外にして送信する
        .content_type("text/plain")
        // invalid なバイト列をボディとして設定する
        .bytes(Bytes::from_static(b"invalid-grpc-body"))
        .await;
    // 415 Unsupported Media Type であることを確認する
    resp.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

// ===== Test 4: /connect/bidi/stream — Connect-RPC bidi エンドポイント ===========

// test_connect_bidi_endpoint は POST /connect/bidi/stream に
// application/json で { "session_id": "test" } を送信して 200 を検証する。
// 01_Bidi適合仕様.md §connect_bidi adapter の bidi handshake を確認する。
#[tokio::test]
async fn test_connect_bidi_endpoint() {
    // TestServer を構築する
    let server = new_server();
    // Content-Type: application/json で Connect bidi stream エンドポイントに POST する
    let resp = server
        .post("/connect/bidi/stream")
        // application/json は connect_bidi adapter が許可する Content-Type
        .content_type("application/json")
        // session_id を含む JSON ボディを送信する
        .json(&serde_json::json!({
            // セッション識別子を指定する
            "session_id": "test-session-e2e",
            // conformance_class を明示する（v1_interactive）
            "conformance_class": "v1_interactive"
        }))
        .await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // state が "Done" であることを確認する（BidiSession 完了を示す）
    assert_eq!(
        body["state"].as_str().unwrap_or(""),
        "Done",
        "/connect/bidi/stream の state が 'Done' でなければならない"
    );
    // session_id がエコーバックされていることを確認する
    assert_eq!(
        body["session_id"].as_str().unwrap_or(""),
        "test-session-e2e",
        "/connect/bidi/stream の session_id がエコーバックされなければならない"
    );
    // adapter が connect_bidi であることを確認する
    assert_eq!(
        body["adapter"].as_str().unwrap_or(""),
        "connect_bidi",
        "/connect/bidi/stream の adapter が 'connect_bidi' でなければならない"
    );
}

// ===== Test 5: /post-sse/bidi/up — paired_post_sse UP エンドポイント ===========

// test_paired_post_sse_up は POST /post-sse/bidi/up が 200 を返すことを検証する。
// 01_Bidi適合仕様.md §paired_post_sse adapter の UP 方向受信を確認する。
#[tokio::test]
async fn test_paired_post_sse_up() {
    // TestServer を構築する
    let server = new_server();
    // POST /post-sse/bidi/up に JSON ボディを送信する
    let resp = server
        .post("/post-sse/bidi/up")
        // Content-Type: application/json を設定する
        .content_type("application/json")
        // session_id と request_id と payload を含む JSON ボディを送信する
        .json(&serde_json::json!({
            // セッション識別子を指定する
            "session_id": "e2e-test-session",
            // UP ストリームと DOWN ストリームを対応させる request_id
            "request_id": "e2e-request-001",
            // クライアントから送信するメッセージペイロード
            "payload": { "msg": "hello from e2e test" }
        }))
        .await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // received フラグが true であることを確認する
    assert_eq!(
        body["received"].as_bool().unwrap_or(false),
        true,
        "/post-sse/bidi/up の received が true でなければならない"
    );
    // session_id がエコーバックされていることを確認する
    assert_eq!(
        body["session_id"].as_str().unwrap_or(""),
        "e2e-test-session",
        "/post-sse/bidi/up の session_id がエコーバックされなければならない"
    );
}

// ===== Test 6: /long-poll/poll — long-poll エンドポイント ======================

// test_long_poll_endpoint は POST /long-poll/poll に { "session_id": "test", "timeout_ms": 100 } を
// 送信して next_resume_token フィールドが存在することを検証する。
// 01_Bidi適合仕様.md §long_poll adapter の SESSION_ORDERED resume_token を確認する。
#[tokio::test]
async fn test_long_poll_endpoint() {
    // TestServer を構築する
    let server = new_server();
    // POST /long-poll/poll に timeout_ms=100 (ms) で送信する（テスト高速化のため短く設定する）
    let resp = server
        .post("/long-poll/poll")
        // Content-Type: application/json を設定する
        .content_type("application/json")
        // session_id と短い timeout_ms を指定する
        .json(&serde_json::json!({
            // セッション識別子を指定する
            "session_id": "e2e-long-poll-session",
            // 100ms のタイムアウトを指定する（テストを素早く完了させる）
            "timeout_ms": 100
        }))
        .await;
    // 200 OK または 204 No Content のどちらかであることを確認する
    // （timeout_ms=100ms の場合は即時 timeout して空 events が返る可能性がある）
    let status = resp.status_code();
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "/long-poll/poll は 200 または 204 を返さなければならない（実際: {}）",
        status
    );
    // long_poll handler は 204 でも JSON body を返す設計であることを確認する
    let body: Value = resp.json();
    // next_resume_token フィールドが存在して空でないことを確認する
    assert!(
        body["next_resume_token"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
        "/long-poll/poll の next_resume_token が空でない文字列でなければならない"
    );
    // session_id がエコーバックされていることを確認する
    assert_eq!(
        body["session_id"].as_str().unwrap_or(""),
        "e2e-long-poll-session",
        "/long-poll/poll の session_id がエコーバックされなければならない"
    );
}

// ===== Test 7: /webtransport/check — WebTransport 利用可否チェック ==============

// test_webtransport_check は GET /webtransport/check が
// 200 と { "available": false } を返すことを検証する。
// 01_Bidi適合仕様.md §web_transport adapter: requires_fallback=true の動作を確認する。
#[tokio::test]
async fn test_webtransport_check() {
    // TestServer を構築する
    let server = new_server();
    // GET /webtransport/check を送信する
    let resp = server.get("/webtransport/check").await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // available フィールドが false であることを確認する（h3_quic 未構成環境）
    assert_eq!(
        body["available"].as_bool().unwrap_or(true),
        false,
        "/webtransport/check の available が false でなければならない（h3_quic_not_configured）"
    );
    // fallback フィールドが存在することを確認する（paired_post_sse に誘導する）
    assert!(
        body["fallback"].is_string(),
        "/webtransport/check の fallback フィールドが文字列でなければならない"
    );
}

// ===== Test 8: /negotiate — capability negotiation エンドポイント ==============

// test_capability_negotiation は GET /negotiate?conformance_class=v1_interactive が
// 200 と { "chosen_adapter_name": "..." } を返すことを検証する。
// 01_Bidi適合仕様.md §Capability Negotiation アルゴリズムの選択結果を確認する。
#[tokio::test]
async fn test_capability_negotiation() {
    // TestServer を構築する
    let server = new_server();
    // GET /negotiate?conformance_class=v1_interactive を送信する
    let resp = server
        .get("/negotiate")
        // クエリパラメータ: v1_interactive を要求する
        .add_query_param("conformance_class", "v1_interactive")
        .await;
    // 200 OK であることを確認する
    resp.assert_status_ok();
    // レスポンスボディを JSON としてパースする
    let body: Value = resp.json();
    // chosen_adapter_name フィールドが存在して空でないことを確認する
    let chosen = body["chosen_adapter_name"].as_str().unwrap_or("");
    assert!(
        !chosen.is_empty(),
        "/negotiate の chosen_adapter_name が空でない文字列でなければならない"
    );
    // chosen_adapter_name が既知の adapter 名であることを確認する
    // 01_Bidi適合仕様.md §adapter↔class supports 対応 の 8 adapter から選択されるべき
    let known_adapters = [
        "grpc_native",
        "connect_bidi",
        "web_transport",
        "sse_paired",
        "paired_post_sse",
        "long_poll",
        "webhook",
        "messaging_bridge",
    ];
    assert!(
        known_adapters.contains(&chosen),
        "/negotiate の chosen_adapter_name が既知 adapter 名でなければならない（実際: {}）",
        chosen
    );
    // fallback_used フィールドが bool であることを確認する
    assert!(
        body["fallback_used"].is_boolean(),
        "/negotiate の fallback_used が bool でなければならない"
    );
}
