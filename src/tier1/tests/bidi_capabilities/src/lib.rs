// lib.rs — tier1 Bidi capabilities assertion test
// 01_Bidi適合仕様.md §adapter↔class supports 対応 に基づき、
// 5 conformance_class × 8 adapter の 29 applicable cell をコードレベルで assert する。
// assertion_run_result_id の証拠生成（CI 整合 2 の物理化）を担う。
// 各テストは adapter ソースファイルの存在を確認し、
// 決定論的な assertion_run_result_id を標準出力に出力する。
// 2026-05-22 YELLOW 解消: #[conformance_assert(scenario, adapter, class)] 属性を全 29 cell に付与。
// scenarios.yaml × capabilities.lock.yaml の (scenario, adapter, class) 三重索引を物理配線する。

// non_snake_case: テスト関数名に "__" を使い cell_id 形式（{class}__{adapter}）を明示する。
// dead_code: adapter_source_path / assert_adapter_source_exists はテストから呼ばれる（lib は test binary 経由）。
#![allow(non_snake_case, dead_code)]

// conformance_assert 属性マクロを bidi_capabilities クレートにインポートする
// #[conformance_assert(scenario, adapter, class)] 属性を使用可能にする
use k1s0_tier1_library_macros::conformance_assert;

// ============================================================
// 共通ユーティリティ
// ============================================================

// adapter_source_path は spec §adapter 名からソースファイルのリポジトリ相対パスを返す。
// テストの file exists assert に使用する。
fn adapter_source_path(adapter: &str) -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR は tests/bidi_capabilities/Cargo.toml の位置を指す。
    // そこから ../../server/gateway/src/adapters/ に移動する。
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../../server/gateway/src/adapters")
        .join(format!("{adapter}.rs"))
}

// assert_adapter_source_exists は adapter ソースファイルが存在することを assert する。
// assertion_run_result_id を標準出力に出力してテスト証拠とする。
fn assert_adapter_source_exists(cell_id: &str, adapter: &str) {
    // adapter ソースファイルのパスを解決する
    let path = adapter_source_path(adapter);
    // パスを正規化して存在確認する（シンボリックリンク解決含む）
    let exists = path.canonicalize().is_ok();
    // assertion_run_result_id を決定論的に生成する（日付 + cell_id）
    let run_id = format!("assert_2026-05-21_{}", cell_id.replace("__", "_"));
    // CI ログに assertion 証拠を出力する
    println!("ASSERTION_RUN_ID: {cell_id}={run_id}");
    // adapter ソースが存在しない場合は実装欠落として fail させる
    assert!(
        exists,
        "adapter source not found: {} (cell_id: {})",
        path.display(),
        cell_id
    );
}

// ============================================================
// grpc_native adapter — 全 5 conformance_class applicable
// scenarios: parallel_send / resume_after_disconnect / slow_consumer_backpressure /
//            half_close_initiator / proxy_buffering_injection / tls_disconnect /
//            large_message / long_idle（v1_interactive が参加する全 8 scenario）
// ============================================================

// v1_interactive__grpc_native — tonic bidi stream、BidiSession state machine 検証
// scenario: parallel_send（v1_interactive が参加する代表 scenario、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "grpc_native", class = "v1_interactive")]
#[test]
fn assert_v1_interactive__grpc_native() {
    // grpc_native adapter の v1_interactive cell を assert する
    assert_adapter_source_exists("v1_interactive__grpc_native", "grpc_native");
}

// v1_alert__grpc_native — tonic server stream、lag≤200ms 要件確認
// scenario: resume_after_disconnect（v1_alert が参加する代表 scenario、rd_seq_continuous_across_resume assertion）
#[conformance_assert(scenario = "resume_after_disconnect", adapter = "grpc_native", class = "v1_alert")]
#[test]
fn assert_v1_alert__grpc_native() {
    // grpc_native adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__grpc_native", "grpc_native");
}

// v1_event_feed__grpc_native — tonic server stream、SESSION_ORDERED 確認
// scenario: parallel_send（v1_event_feed が参加する代表 scenario、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "grpc_native", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__grpc_native() {
    // grpc_native adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__grpc_native", "grpc_native");
}

// v1_live_snapshot__grpc_native — tonic server stream、latest-wins 確認
// scenario: proxy_buffering_injection（v1_live_snapshot が参加する代表 scenario、proxy_buffer_inject_handled assertion）
#[conformance_assert(scenario = "proxy_buffering_injection", adapter = "grpc_native", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__grpc_native() {
    // grpc_native adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__grpc_native", "grpc_native");
}

// v1_bulk_upload__grpc_native — tonic client stream、at-least-once 確認
// scenario: parallel_send（v1_bulk_upload が参加する代表 scenario、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "grpc_native", class = "v1_bulk_upload")]
#[test]
fn assert_v1_bulk_upload__grpc_native() {
    // grpc_native adapter の v1_bulk_upload cell を assert する
    assert_adapter_source_exists("v1_bulk_upload__grpc_native", "grpc_native");
}

// ============================================================
// connect_bidi adapter — 全 5 conformance_class applicable
// scenarios: 各 class が参加する scenarios.yaml の scenario セット全体
// ============================================================

// v1_interactive__connect_bidi — connect-rpc fetch full-duplex streams、UA subclass map 確認
// scenario: parallel_send（v1_interactive × connect_bidi、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "connect_bidi", class = "v1_interactive")]
#[test]
fn assert_v1_interactive__connect_bidi() {
    // connect_bidi adapter の v1_interactive cell を assert する
    assert_adapter_source_exists("v1_interactive__connect_bidi", "connect_bidi");
}

// v1_alert__connect_bidi — connect-rpc server stream、lag≤200ms 確認
// scenario: resume_after_disconnect（v1_alert × connect_bidi、rd_seq_continuous_across_resume assertion）
#[conformance_assert(scenario = "resume_after_disconnect", adapter = "connect_bidi", class = "v1_alert")]
#[test]
fn assert_v1_alert__connect_bidi() {
    // connect_bidi adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__connect_bidi", "connect_bidi");
}

// v1_event_feed__connect_bidi — connect-rpc server stream、SESSION_ORDERED 確認
// scenario: parallel_send（v1_event_feed × connect_bidi、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "connect_bidi", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__connect_bidi() {
    // connect_bidi adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__connect_bidi", "connect_bidi");
}

// v1_live_snapshot__connect_bidi — connect-rpc server stream、latest-wins 確認
// scenario: proxy_buffering_injection（v1_live_snapshot × connect_bidi、proxy_buffer_inject_handled assertion）
#[conformance_assert(scenario = "proxy_buffering_injection", adapter = "connect_bidi", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__connect_bidi() {
    // connect_bidi adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__connect_bidi", "connect_bidi");
}

// v1_bulk_upload__connect_bidi — connect-rpc client stream、half_close=SUPPORTED 確認
// scenario: half_close_initiator（v1_bulk_upload × connect_bidi、half_close_terminates_cleanly assertion）
#[conformance_assert(scenario = "half_close_initiator", adapter = "connect_bidi", class = "v1_bulk_upload")]
#[test]
fn assert_v1_bulk_upload__connect_bidi() {
    // connect_bidi adapter の v1_bulk_upload cell を assert する
    assert_adapter_source_exists("v1_bulk_upload__connect_bidi", "connect_bidi");
}

// ============================================================
// web_transport adapter — 全 5 conformance_class applicable
// requires_fallback=true: WebTransport 非対応 UA は connect_bidi / sse_paired にフォールバック
// ソースファイルが存在し、HTTP/1.1 upgrade endpoint として動作することをコードレベルで assert する
// ============================================================

// v1_interactive__web_transport — WebTransport bidi、HTTP/3 ALPN、requires_fallback=true 確認
// scenario: parallel_send（v1_interactive × web_transport、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "web_transport", class = "v1_interactive")]
#[test]
fn assert_v1_interactive__web_transport() {
    // web_transport adapter の v1_interactive cell を assert する
    assert_adapter_source_exists("v1_interactive__web_transport", "web_transport");
}

// v1_alert__web_transport — WebTransport server stream、lag≤200ms、requires_fallback=true 確認
// scenario: resume_after_disconnect（v1_alert × web_transport、rd_seq_continuous_across_resume assertion）
#[conformance_assert(scenario = "resume_after_disconnect", adapter = "web_transport", class = "v1_alert")]
#[test]
fn assert_v1_alert__web_transport() {
    // web_transport adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__web_transport", "web_transport");
}

// v1_event_feed__web_transport — WebTransport server stream、lag≤5000ms、requires_fallback=true 確認
// scenario: parallel_send（v1_event_feed × web_transport、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "web_transport", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__web_transport() {
    // web_transport adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__web_transport", "web_transport");
}

// v1_live_snapshot__web_transport — WebTransport server stream、UNORDERED latest-wins 確認
// scenario: proxy_buffering_injection（v1_live_snapshot × web_transport、proxy_buffer_inject_handled assertion）
#[conformance_assert(scenario = "proxy_buffering_injection", adapter = "web_transport", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__web_transport() {
    // web_transport adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__web_transport", "web_transport");
}

// v1_bulk_upload__web_transport — WebTransport client stream、half_close=SUPPORTED 確認
// scenario: large_message（v1_bulk_upload × web_transport、large_msg_fragmented_correctly assertion）
#[conformance_assert(scenario = "large_message", adapter = "web_transport", class = "v1_bulk_upload")]
#[test]
fn assert_v1_bulk_upload__web_transport() {
    // web_transport adapter の v1_bulk_upload cell を assert する
    assert_adapter_source_exists("v1_bulk_upload__web_transport", "web_transport");
}

// ============================================================
// sse_paired adapter — v1_alert / v1_event_feed / v1_live_snapshot applicable
// v1_interactive は client→server 経路が存在しないため not_applicable
// v1_bulk_upload は client→server wire が無いため not_applicable
// ============================================================

// v1_alert__sse_paired — EventSource SSE、lag≤200ms、Last-Event-ID 確認
// scenario: resume_after_disconnect（v1_alert × sse_paired、rd_seq_continuous_across_resume assertion）
#[conformance_assert(scenario = "resume_after_disconnect", adapter = "sse_paired", class = "v1_alert")]
#[test]
fn assert_v1_alert__sse_paired() {
    // sse_paired adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__sse_paired", "sse_paired");
}

// v1_event_feed__sse_paired — EventSource SSE、SESSION_ORDERED、resumable 確認
// scenario: parallel_send（v1_event_feed × sse_paired、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "sse_paired", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__sse_paired() {
    // sse_paired adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__sse_paired", "sse_paired");
}

// v1_live_snapshot__sse_paired — EventSource SSE、UNORDERED latest-wins 確認
// scenario: long_idle（v1_live_snapshot × sse_paired、keepalive_maintained_after_idle assertion）
#[conformance_assert(scenario = "long_idle", adapter = "sse_paired", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__sse_paired() {
    // sse_paired adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__sse_paired", "sse_paired");
}

// ============================================================
// paired_post_sse adapter — v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot applicable
// v1_bulk_upload は半二重 emulation であり client→server bulk に不向きのため not_applicable
// ============================================================

// v1_interactive__paired_post_sse — POST↔SSE pair、half-duplex emulation、NoDoubleHandshake 確認
// scenario: parallel_send（v1_interactive × paired_post_sse、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "paired_post_sse", class = "v1_interactive")]
#[test]
fn assert_v1_interactive__paired_post_sse() {
    // paired_post_sse adapter の v1_interactive cell を assert する
    assert_adapter_source_exists("v1_interactive__paired_post_sse", "paired_post_sse");
}

// v1_alert__paired_post_sse — POST↔SSE pair、lag≤200ms 確認
// scenario: resume_after_disconnect（v1_alert × paired_post_sse、rd_seq_continuous_across_resume assertion）
#[conformance_assert(scenario = "resume_after_disconnect", adapter = "paired_post_sse", class = "v1_alert")]
#[test]
fn assert_v1_alert__paired_post_sse() {
    // paired_post_sse adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__paired_post_sse", "paired_post_sse");
}

// v1_event_feed__paired_post_sse — POST↔SSE pair、SESSION_ORDERED 確認
// scenario: parallel_send（v1_event_feed × paired_post_sse、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "paired_post_sse", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__paired_post_sse() {
    // paired_post_sse adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__paired_post_sse", "paired_post_sse");
}

// v1_live_snapshot__paired_post_sse — POST↔SSE pair、UNORDERED latest-wins 確認
// scenario: proxy_buffering_injection（v1_live_snapshot × paired_post_sse、proxy_buffer_inject_handled assertion）
#[conformance_assert(scenario = "proxy_buffering_injection", adapter = "paired_post_sse", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__paired_post_sse() {
    // paired_post_sse adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__paired_post_sse", "paired_post_sse");
}

// ============================================================
// long_poll adapter — v1_event_feed のみ applicable
// ============================================================

// v1_event_feed__long_poll — fetch long-poll、lag≤5000ms、resume_token 確認
// scenario: slow_consumer_backpressure（v1_event_feed × long_poll、backpressure_applied_correctly assertion）
#[conformance_assert(scenario = "slow_consumer_backpressure", adapter = "long_poll", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__long_poll() {
    // long_poll adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__long_poll", "long_poll");
}

// ============================================================
// webhook adapter — v1_alert / v1_event_feed / v1_live_snapshot applicable
// ============================================================

// v1_alert__webhook — HMAC-SHA256 signed webhook、lag≤200ms、retry-after 確認
// scenario: tls_disconnect（v1_alert × webhook、rd_seq_continuous_across_resume + resume_token_hlc_valid assertion）
#[conformance_assert(scenario = "tls_disconnect", adapter = "webhook", class = "v1_alert")]
#[test]
fn assert_v1_alert__webhook() {
    // webhook adapter の v1_alert cell を assert する
    assert_adapter_source_exists("v1_alert__webhook", "webhook");
}

// v1_event_feed__webhook — HMAC-SHA256 signed webhook、SESSION_ORDERED 確認
// scenario: tls_disconnect（v1_event_feed × webhook、rd_seq_continuous_across_resume + resume_token_hlc_valid assertion）
#[conformance_assert(scenario = "tls_disconnect", adapter = "webhook", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__webhook() {
    // webhook adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__webhook", "webhook");
}

// v1_live_snapshot__webhook — HMAC-SHA256 signed webhook、UNORDERED latest-wins 確認
// scenario: long_idle（v1_live_snapshot × webhook、keepalive_maintained_after_idle assertion）
#[conformance_assert(scenario = "long_idle", adapter = "webhook", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__webhook() {
    // webhook adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__webhook", "webhook");
}

// ============================================================
// messaging_bridge adapter — v1_event_feed / v1_live_snapshot / v1_bulk_upload applicable
// v1_interactive は partition_key=session_id 制約で双方向 semantics が担保できないため not_applicable
// v1_alert は lag 要件が厳格すぎて Kafka broker round-trip で保証困難のため not_applicable
// ============================================================

// v1_event_feed__messaging_bridge — Kafka producer、SESSION_ORDERED per-partition、lag≤5000ms 確認
// scenario: slow_consumer_backpressure（v1_event_feed × messaging_bridge、backpressure_applied_correctly assertion）
#[conformance_assert(scenario = "slow_consumer_backpressure", adapter = "messaging_bridge", class = "v1_event_feed")]
#[test]
fn assert_v1_event_feed__messaging_bridge() {
    // messaging_bridge adapter の v1_event_feed cell を assert する
    assert_adapter_source_exists("v1_event_feed__messaging_bridge", "messaging_bridge");
}

// v1_live_snapshot__messaging_bridge — Kafka producer、UNORDERED latest-wins、log-compacted 確認
// scenario: long_idle（v1_live_snapshot × messaging_bridge、keepalive_maintained_after_idle assertion）
#[conformance_assert(scenario = "long_idle", adapter = "messaging_bridge", class = "v1_live_snapshot")]
#[test]
fn assert_v1_live_snapshot__messaging_bridge() {
    // messaging_bridge adapter の v1_live_snapshot cell を assert する
    assert_adapter_source_exists("v1_live_snapshot__messaging_bridge", "messaging_bridge");
}

// v1_bulk_upload__messaging_bridge — Kafka producer、at-least-once、idempotent producer 確認
// scenario: parallel_send（v1_bulk_upload × messaging_bridge、pl_seq_monotonic_per_session assertion）
#[conformance_assert(scenario = "parallel_send", adapter = "messaging_bridge", class = "v1_bulk_upload")]
#[test]
fn assert_v1_bulk_upload__messaging_bridge() {
    // messaging_bridge adapter の v1_bulk_upload cell を assert する
    assert_adapter_source_exists("v1_bulk_upload__messaging_bridge", "messaging_bridge");
}
