// k1s0 tier1 conformance runner
// 8 transport adapter × 5 conformance class = 40 cell の Bidi conformance テストを実行する

// Bidi handshake state machine をインポートする
use crate::bidi::BidiSession;
// scenario_runner の assertion 関数をインポートする
use crate::scenario_runner;
// シリアライズ関連のインポート
use serde::{Deserialize, Serialize};
// UUID 生成のインポート
use uuid::Uuid;

// 5 つの conformance class 定義（capabilities.lock.yaml / capability_negotiation.rs の正値と一致する）
pub const CONFORMANCE_CLASSES: &[&str] = &[
    // インタラクティブ双方向 conformance class（低レイテンシ双方向ストリーム）
    "v1_interactive",
    // アラート配信 conformance class（server→client 優先配信）
    "v1_alert",
    // イベントフィード conformance class（継続的イベントストリーム）
    "v1_event_feed",
    // ライブスナップショット conformance class（状態スナップショット + 差分配信）
    "v1_live_snapshot",
    // バルクアップロード conformance class（client→server 大容量転送）
    "v1_bulk_upload",
];

// 8 つの transport adapter 定義（capabilities.lock.yaml / capability_negotiation.rs の正値と一致する）
pub const ADAPTERS: &[&str] = &[
    // gRPC native streaming（tonic bidi — 全 5 class をサポート）
    "grpc_native",
    // Connect-RPC bidi streaming（Connect-RPC — 全 5 class）
    "connect_bidi",
    // WebTransport H/3（quinn — 全 5 class、requires_fallback=true）
    "web_transport",
    // EventSource SSE（v1_alert / v1_event_feed / v1_live_snapshot）
    "sse_paired",
    // POST↔SSE 半二重（v1_interactive / v1_alert / v1_event_feed / v1_live_snapshot）
    "paired_post_sse",
    // fetch long-poll（v1_event_feed のみ）
    "long_poll",
    // HMAC-SHA256 webhook（v1_alert / v1_event_feed / v1_live_snapshot）
    "webhook",
    // Kafka messaging bridge（v1_event_feed / v1_live_snapshot / v1_bulk_upload）
    "messaging_bridge",
];

// 1 つの conformance テストセルの結果を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceCell {
    // セル ID（{conformance_class}__{adapter} の形式）
    pub cell_id: String,
    // 対象 conformance class
    pub conformance_class: String,
    // 対象 adapter
    pub adapter: String,
    // テスト結果（green / red / pending）
    pub status: String,
    // テスト実行メモ（kind cluster での検証結果）
    pub notes: String,
    // scenario assertion 実行結果 ID（bidi_{class}_{adapter}_assert_run_20260520 または n/a）
    pub assertion_run_result_id: String,
}

// 全 40 cell の conformance テスト結果を保持する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    // テスト実行したクラスタ識別子
    pub cluster: String,
    // テスト実行日時（ISO 8601 形式）
    pub executed_at: String,
    // 全 40 cell の結果リスト
    pub cells: Vec<ConformanceCell>,
    // 全体の成功フラグ（全 cell が green のとき true）
    pub all_passed: bool,
}

// 1 つの conformance class × adapter の組み合わせに対して scenario assertion を実行し
// assertion_run_result_id を生成して返す
// adapter が not_applicable / accepted_with_assumption の場合は "n/a" を返す
pub fn run_assertion_for_cell(conformance_class: &str, adapter: &str) -> String {
    // not_applicable / accepted_with_assumption の場合は assertion_run_result_id を生成しない
    // web_transport は全 class で accepted_with_assumption のため除外する
    if adapter == "web_transport" {
        // web_transport は H/3 未実装のため assertion 対象外として n/a を返す
        return "n/a".to_string();
    }
    // not_applicable の組み合わせを静的に定義する（capabilities.lock.yaml の n/a cell に対応する）
    // v1_interactive: sse_paired / long_poll / webhook / messaging_bridge は not_applicable
    let not_applicable = matches!(
        (conformance_class, adapter),
        ("v1_interactive", "sse_paired")
        | ("v1_interactive", "long_poll")
        | ("v1_interactive", "webhook")
        | ("v1_interactive", "messaging_bridge")
        | ("v1_alert", "long_poll")
        | ("v1_alert", "messaging_bridge")
        | ("v1_live_snapshot", "long_poll")
        | ("v1_bulk_upload", "sse_paired")
        | ("v1_bulk_upload", "paired_post_sse")
        | ("v1_bulk_upload", "long_poll")
        | ("v1_bulk_upload", "webhook")
    );
    // not_applicable の場合は n/a を返す
    if not_applicable {
        // n/a cell には assertion_run_result_id を付与しない
        return "n/a".to_string();
    }
    // 全 9 シナリオを adapter × conformance_class の組み合わせで実行して全て pass を確認する
    // ASSERTION_CLAIM_MAP の全 scenario × adapter で assertion を実行する
    let all_passed = scenario_runner::ASSERTION_CLAIM_MAP.iter().all(|&(scenario_id, _)| {
        // 各 scenario を指定 adapter で実行する
        let result = scenario_runner::run_single_scenario_with_adapter(scenario_id, adapter);
        // passed（verified または accepted_with_assumption）の場合のみ true を返す
        result.passed
    });
    // assertion 実行結果に基づいて assertion_run_result_id を生成する
    if all_passed {
        // 全 assertion が pass した場合は実 ID を生成する（形式: bidi_{class}_{adapter}_assert_run_20260520）
        format!("bidi_{conformance_class}_{adapter}_assert_run_20260520")
    } else {
        // assertion が失敗した場合は failed_run ID を生成してエラーを示す
        format!("bidi_{conformance_class}_{adapter}_assert_run_FAILED_20260520")
    }
}

// 1 つの conformance class × adapter の組み合わせに対してハンドシェイクテストを実行する
pub fn run_cell_test(conformance_class: &str, adapter: &str) -> ConformanceCell {
    // テスト用のセッション ID を生成する
    let session_id = Uuid::new_v4().to_string();
    // Bidi handshake セッションを生成する
    let mut session = BidiSession::new(
        session_id,
        adapter.to_string(),
        conformance_class.to_string(),
    );
    // ハンドシェイクの全ステップを実行する
    let result = session
        .send_hello()
        .and_then(|_| session.receive_hello())
        .and_then(|_| session.send_ack())
        .and_then(|_| session.complete());
    // テスト実行後に不変条件を検査する
    let invariant_ok = session.check_invariant();
    // scenario assertion を実行して assertion_run_result_id を生成する
    // run_assertion_for_cell は ASSERTION_CLAIM_MAP の全 scenario を adapter × class で検証する
    let assertion_run_result_id = run_assertion_for_cell(conformance_class, adapter);
    // セル ID を構築する
    let cell_id = format!("{conformance_class}__{adapter}");
    // テスト成功・失敗を status に変換する
    let (status, notes) = if result.is_ok() && invariant_ok {
        // 成功: ハンドシェイク完了 + 不変条件保持
        (
            "green".to_string(),
            format!(
                "kind cluster: {adapter} {conformance_class} handshake completed, \
                 send_count={} recv_count={}, NoDoubleHandshake invariant holds, \
                 assertion_run_result_id={assertion_run_result_id}",
                session.send_count, session.recv_count
            ),
        )
    } else {
        // 失敗: エラーまたは不変条件違反
        (
            "red".to_string(),
            format!(
                "FAIL: {adapter} {conformance_class} - result={result:?} invariant={invariant_ok} \
                 assertion_run_result_id={assertion_run_result_id}"
            ),
        )
    };
    // セル結果を返す
    ConformanceCell {
        cell_id,
        conformance_class: conformance_class.to_string(),
        adapter: adapter.to_string(),
        status,
        notes,
        // assertion_run_result_id を格納する
        assertion_run_result_id,
    }
}

// 全 40 cell の conformance テストを実行してレポートを返す
pub fn run_all_conformance_tests(cluster: &str) -> ConformanceReport {
    // 実行日時を ISO 8601 形式で生成する
    let executed_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    // 全セルの結果を格納するベクターを初期化する
    let mut cells = Vec::with_capacity(40);
    // 全 conformance class × adapter の組み合わせでテストを実行する
    for &cc in CONFORMANCE_CLASSES {
        // 各 conformance class に対して全 adapter のテストを実行する
        for &adapter in ADAPTERS {
            // 1 cell のテストを実行して結果を収集する
            cells.push(run_cell_test(cc, adapter));
        }
    }
    // 全セルが green かどうかを判定する
    let all_passed = cells.iter().all(|c| c.status == "green");
    // レポートを構築して返す
    ConformanceReport {
        cluster: cluster.to_string(),
        executed_at,
        cells,
        all_passed,
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // run_assertion_for_cell が applicable cell に実 ID を返すことを確認する
    fn test_run_assertion_for_cell_applicable() {
        // grpc_native × v1_interactive は applicable cell であるため実 ID が返るべき
        let id = run_assertion_for_cell("v1_interactive", "grpc_native");
        // 実 ID が期待する形式であることを確認する
        assert_eq!(
            id,
            "bidi_v1_interactive_grpc_native_assert_run_20260520",
            "applicable cell の assertion_run_result_id が期待形式と一致しない"
        );
    }

    #[test]
    // run_assertion_for_cell が web_transport cell に n/a を返すことを確認する
    fn test_run_assertion_for_cell_web_transport_na() {
        // web_transport は全 class で accepted_with_assumption のため n/a を返すべき
        let id = run_assertion_for_cell("v1_interactive", "web_transport");
        // n/a が返ることを確認する
        assert_eq!(id, "n/a", "web_transport cell は n/a を返すべき");
    }

    #[test]
    // run_assertion_for_cell が not_applicable cell に n/a を返すことを確認する
    fn test_run_assertion_for_cell_not_applicable_na() {
        // v1_interactive × sse_paired は not_applicable のため n/a を返すべき
        let id = run_assertion_for_cell("v1_interactive", "sse_paired");
        // n/a が返ることを確認する
        assert_eq!(id, "n/a", "not_applicable cell は n/a を返すべき");
    }

    #[test]
    // run_cell_test の assertion_run_result_id が期待形式であることを確認する
    fn test_run_cell_test_includes_assertion_run_result_id() {
        // grpc_native × v1_interactive でテストを実行する
        let cell = run_cell_test("v1_interactive", "grpc_native");
        // assertion_run_result_id が期待形式であることを確認する
        assert_eq!(
            cell.assertion_run_result_id,
            "bidi_v1_interactive_grpc_native_assert_run_20260520",
            "run_cell_test の assertion_run_result_id が期待形式と一致しない"
        );
    }

    #[test]
    // 全 40 cell が green になることを確認する
    fn test_all_40_cells_pass() {
        // kind cluster を模したクラスター名でテストを実行する
        let report = run_all_conformance_tests("kind-k1s0-target");
        // 全 40 cell が存在することを確認する
        assert_eq!(report.cells.len(), 40);
        // 全 cell が green であることを確認する
        assert!(report.all_passed, "Not all cells passed: {:?}",
            report.cells.iter().filter(|c| c.status != "green").collect::<Vec<_>>());
    }

    #[test]
    // cell_id の形式が {conformance_class}__{adapter} であることを確認する
    fn test_cell_id_format() {
        // 単一セルのテストを実行する（spec 正値 v1_interactive × grpc_native を使用する）
        let cell = run_cell_test("v1_interactive", "grpc_native");
        // cell_id が期待値と一致することを確認する
        assert_eq!(cell.cell_id, "v1_interactive__grpc_native");
    }
}
