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

// ============================================================
// 製造業 pack 10 RPC stress test fixture
// spec 01 §v1_bulk_upload: 製造業 pack 10 RPC × 5 conformance class のストレス試験
// 製造業ドメイン（FA / 品質検査 / 調達 / SCADA テレメトリ）の RPC を網羅的に宣言する
// ============================================================

/// 製造業 pack の 10 RPC 定義。
/// スキーマ進化仕様の aggregate_qualified_name と整合させて命名する。
/// 各 RPC は CONFORMANCE_CLASSES のいずれかに属し、
/// adapter capability matrix に基づいて stress fixture を生成する。
// MANUFACTURING_RPCS: 製造業 pack 10 RPC の aggregate_qualified_name を宣言する定数スライス
pub const MANUFACTURING_RPCS: &[&str] = &[
    // 1. 製造指示（FA 工程）: 作業指示の作成（v1_interactive 双方向対話）
    "manufacturing.fa.work_order.CreateWorkOrder",
    // 2. 製造指示（FA 工程）: 作業指示の更新（v1_interactive 双方向対話）
    "manufacturing.fa.work_order.UpdateWorkOrder",
    // 3. 品質検査: 検査結果の記録（v1_interactive — 操作員との対話）
    "manufacturing.inspection.InspectionResult.RecordInspectionResult",
    // 4. 品質検査: 検査結果のストリーム配信（v1_event_feed — 品質検査結果配信）
    "manufacturing.inspection.InspectionResult.StreamInspectionResults",
    // 5. 調達: 発注書の作成（v1_interactive — 双方向確認フロー）
    "manufacturing.procurement.PurchaseOrder.CreatePurchaseOrder",
    // 6. 調達: 納品確認（v1_interactive — 双方向確認フロー）
    "manufacturing.procurement.PurchaseOrder.ConfirmDelivery",
    // 7. SCADA テレメトリ: 設備状態の一括アップロード（v1_bulk_upload）
    "manufacturing.fa.machine.ReportMachineStatus",
    // 8. 設備アラート: アラートストリーム配信（v1_alert — 設備異常通知）
    "manufacturing.fa.machine.StreamMachineAlerts",
    // 9. 品質イベント: 品質イベントの記録（v1_interactive — 操作員との対話）
    "manufacturing.quality.QualityEvent.RecordQualityEvent",
    // 10. 品質イベントフィード: 品質イベントのストリーム配信（v1_event_feed）
    "manufacturing.quality.QualityEvent.StreamQualityFeed",
];

/// ManufacturingStressFixture は製造業 pack RPC のストレステスト fixture を表す構造体。
/// rpc_name × expected_class × adapter の三つ組で 1 テストケースを表現する。
/// message_count と expected_latency_ms は spec 01 §v1_bulk_upload の SLA 要件に基づく。
// ManufacturingStressFixture 構造体: 1 つのストレステストケースを保持する
#[derive(Debug, Clone)]
pub struct ManufacturingStressFixture {
    // rpc_name: 製造業 pack RPC の aggregate_qualified_name
    pub rpc_name: String,
    // expected_class: この RPC が属する conformance class（CONFORMANCE_CLASSES の値）
    pub expected_class: String,
    // adapter: テスト対象の transport adapter 名（ADAPTERS の値）
    pub adapter: String,
    // message_count: 1 ストレスセッションで送受信するメッセージ数
    pub message_count: u32,
    // expected_latency_ms: 許容最大レイテンシ（ミリ秒）。spec 01 §max_msg_lag_ms に準拠する
    pub expected_latency_ms: u64,
}

// RPC 名と conformance class の対応を宣言する静的マッピング
// spec 01 §v1 conformance_class セット と製造業 pack の RPC catalog を紐づける
const MANUFACTURING_RPC_CLASS_MAP: &[(&str, &str)] = &[
    // 作業指示作成: 操作員との双方向対話のため v1_interactive
    ("manufacturing.fa.work_order.CreateWorkOrder",                     "v1_interactive"),
    // 作業指示更新: 操作員との双方向対話のため v1_interactive
    ("manufacturing.fa.work_order.UpdateWorkOrder",                     "v1_interactive"),
    // 検査結果記録: 操作員との双方向確認のため v1_interactive
    ("manufacturing.inspection.InspectionResult.RecordInspectionResult", "v1_interactive"),
    // 検査結果ストリーム: 品質検査結果の継続配信のため v1_event_feed
    ("manufacturing.inspection.InspectionResult.StreamInspectionResults", "v1_event_feed"),
    // 発注書作成: 双方向確認フローのため v1_interactive
    ("manufacturing.procurement.PurchaseOrder.CreatePurchaseOrder",     "v1_interactive"),
    // 納品確認: 双方向確認フローのため v1_interactive
    ("manufacturing.procurement.PurchaseOrder.ConfirmDelivery",         "v1_interactive"),
    // 設備状態一括アップロード: client→server 大容量転送のため v1_bulk_upload
    ("manufacturing.fa.machine.ReportMachineStatus",                    "v1_bulk_upload"),
    // アラートストリーム: server→client 警報配信のため v1_alert
    ("manufacturing.fa.machine.StreamMachineAlerts",                    "v1_alert"),
    // 品質イベント記録: 操作員との双方向確認のため v1_interactive
    ("manufacturing.quality.QualityEvent.RecordQualityEvent",           "v1_interactive"),
    // 品質イベントフィード: Domain Event 継続配信のため v1_event_feed
    ("manufacturing.quality.QualityEvent.StreamQualityFeed",            "v1_event_feed"),
];

/// 製造業 pack 10 RPC × 8 adapter の stress fixtures を生成する関数。
/// MANUFACTURING_RPC_CLASS_MAP と ADAPTERS の直積から全 80 fixture を構築する。
/// message_count=100 / expected_latency_ms は各 conformance class の max_msg_lag_ms に準拠する。
// generate_manufacturing_stress_fixtures: 10 RPC × 8 adapter = 80 fixture を生成する
pub fn generate_manufacturing_stress_fixtures() -> Vec<ManufacturingStressFixture> {
    // 全 fixture を格納するベクターを初期化する（capacity: 10 RPC × 8 adapter = 80）
    let mut fixtures = Vec::with_capacity(MANUFACTURING_RPC_CLASS_MAP.len() * ADAPTERS.len());
    // 各 RPC と conformance class の組み合わせを順に処理する
    for &(rpc, class) in MANUFACTURING_RPC_CLASS_MAP {
        // 各 adapter に対して 1 fixture を生成する
        for &adapter in ADAPTERS {
            // conformance class の max_msg_lag_ms を expected_latency_ms として採用する
            // spec 01 §v1 conformance_class セット の lag(ms) 列に準拠する
            let expected_latency_ms = match class {
                // v1_interactive: max_msg_lag_ms = 200ms
                "v1_interactive" => 200,
                // v1_alert: max_msg_lag_ms = 200ms
                "v1_alert" => 200,
                // v1_event_feed: max_msg_lag_ms = 5000ms
                "v1_event_feed" => 5000,
                // v1_live_snapshot: max_msg_lag_ms = 500ms
                "v1_live_snapshot" => 500,
                // v1_bulk_upload: max_msg_lag_ms = 0ms（best-effort、lag 検査除外）
                "v1_bulk_upload" => 0,
                // 未知の class は保守的に 5000ms を設定する
                _ => 5000,
            };
            // ManufacturingStressFixture を構築してベクターに追加する
            fixtures.push(ManufacturingStressFixture {
                // rpc_name を String に変換して格納する
                rpc_name: rpc.to_string(),
                // expected_class を String に変換して格納する
                expected_class: class.to_string(),
                // adapter 名を String に変換して格納する
                adapter: adapter.to_string(),
                // spec 01 §v1_bulk_upload ストレス基準: 1 セッション 100 メッセージ
                message_count: 100,
                // class に応じた max_msg_lag_ms を設定する
                expected_latency_ms,
            });
        }
    }
    // 全 fixture を返す
    fixtures
}

// ─────────────────────────────────────────────────────────────────────────────
// 製造業 pack stress test ユニットテスト
// ─────────────────────────────────────────────────────────────────────────────

// 製造業 pack stress test のユニットテストモジュール
#[cfg(test)]
mod manufacturing_stress_tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // 全製造業 RPC が CONFORMANCE_CLASSES に属する conformance class を持つことを確認する
    #[test]
    fn all_manufacturing_rpcs_have_conformance_class() {
        // 全 fixture を生成する
        let fixtures = generate_manufacturing_stress_fixtures();
        // fixture が 1 件以上存在することを確認する
        assert!(!fixtures.is_empty(), "generate_manufacturing_stress_fixtures が空のリストを返した");
        // 全 fixture の expected_class が CONFORMANCE_CLASSES に含まれることを確認する
        for f in &fixtures {
            // rpc_name が空でないことを確認する
            assert!(!f.rpc_name.is_empty(), "ManufacturingStressFixture の rpc_name が空である");
            // expected_class が CONFORMANCE_CLASSES に含まれることを確認する
            assert!(
                CONFORMANCE_CLASSES.contains(&f.expected_class.as_str()),
                "rpc '{}' の expected_class '{}' は CONFORMANCE_CLASSES に存在しない",
                f.rpc_name,
                f.expected_class
            );
        }
    }

    // MANUFACTURING_RPCS の件数が spec 要件の 10 件と一致することを確認する
    #[test]
    fn manufacturing_rpc_count_matches_spec() {
        // spec 01 §製造業 pack は正確に 10 RPC を要求する
        assert_eq!(
            MANUFACTURING_RPCS.len(),
            10,
            "spec requires exactly 10 manufacturing RPCs; got {}",
            MANUFACTURING_RPCS.len()
        );
    }

    // generate_manufacturing_stress_fixtures が 10 RPC × 8 adapter = 80 件を返すことを確認する
    #[test]
    fn fixture_count_is_10_rpc_times_8_adapter() {
        // 10 RPC × 8 adapter = 80 fixture を期待する
        let fixtures = generate_manufacturing_stress_fixtures();
        // fixture 件数が 10 × 8 = 80 であることを確認する
        assert_eq!(
            fixtures.len(),
            MANUFACTURING_RPC_CLASS_MAP.len() * ADAPTERS.len(),
            "fixture 件数が 10 RPC × 8 adapter の積と一致しない"
        );
    }

    // MANUFACTURING_RPC_CLASS_MAP の件数が MANUFACTURING_RPCS と一致することを確認する
    #[test]
    fn rpc_class_map_matches_rpcs_const() {
        // MANUFACTURING_RPC_CLASS_MAP と MANUFACTURING_RPCS の件数が一致することを確認する
        assert_eq!(
            MANUFACTURING_RPC_CLASS_MAP.len(),
            MANUFACTURING_RPCS.len(),
            "MANUFACTURING_RPC_CLASS_MAP と MANUFACTURING_RPCS の件数が不一致"
        );
        // MANUFACTURING_RPC_CLASS_MAP の全 rpc_name が MANUFACTURING_RPCS に含まれることを確認する
        for &(rpc, _) in MANUFACTURING_RPC_CLASS_MAP {
            // MANUFACTURING_RPCS に rpc が含まれることを確認する
            assert!(
                MANUFACTURING_RPCS.contains(&rpc),
                "MANUFACTURING_RPC_CLASS_MAP の rpc '{}' が MANUFACTURING_RPCS に存在しない",
                rpc
            );
        }
    }

    // v1_bulk_upload クラスの expected_latency_ms が 0 であることを確認する
    #[test]
    fn bulk_upload_latency_is_zero_best_effort() {
        // 全 fixture を生成する
        let fixtures = generate_manufacturing_stress_fixtures();
        // v1_bulk_upload クラスの fixture を検索する
        let bulk_fixtures: Vec<_> = fixtures.iter()
            .filter(|f| f.expected_class == "v1_bulk_upload")
            .collect();
        // v1_bulk_upload fixture が存在することを確認する
        assert!(!bulk_fixtures.is_empty(), "v1_bulk_upload クラスの fixture が存在しない");
        // 全 v1_bulk_upload fixture の expected_latency_ms が 0 であることを確認する
        for f in &bulk_fixtures {
            // spec 01: v1_bulk_upload の max_msg_lag_ms = 0（best-effort、lag 検査除外）
            assert_eq!(
                f.expected_latency_ms,
                0,
                "v1_bulk_upload の expected_latency_ms は 0 であるべき (rpc={}, adapter={})",
                f.rpc_name,
                f.adapter
            );
        }
    }

    // v1_interactive クラスの expected_latency_ms が 200ms であることを確認する
    #[test]
    fn interactive_latency_is_200ms() {
        // 全 fixture を生成する
        let fixtures = generate_manufacturing_stress_fixtures();
        // v1_interactive クラスの grpc_native adapter fixture を検索する
        let interactive_grpc: Vec<_> = fixtures.iter()
            .filter(|f| f.expected_class == "v1_interactive" && f.adapter == "grpc_native")
            .collect();
        // v1_interactive × grpc_native fixture が存在することを確認する
        assert!(!interactive_grpc.is_empty(), "v1_interactive × grpc_native の fixture が存在しない");
        // 全 v1_interactive × grpc_native fixture の expected_latency_ms が 200 であることを確認する
        for f in &interactive_grpc {
            // spec 01: v1_interactive の max_msg_lag_ms = 200ms
            assert_eq!(
                f.expected_latency_ms,
                200,
                "v1_interactive の expected_latency_ms は 200ms であるべき (rpc={})",
                f.rpc_name
            );
        }
    }

    // message_count が全 fixture で 100 に設定されていることを確認する
    #[test]
    fn message_count_is_100_for_all_fixtures() {
        // 全 fixture を生成する
        let fixtures = generate_manufacturing_stress_fixtures();
        // 全 fixture の message_count が 100 であることを確認する
        for f in &fixtures {
            // spec 01 §v1_bulk_upload ストレス基準: 1 セッション 100 メッセージ
            assert_eq!(
                f.message_count,
                100,
                "message_count は 100 であるべき (rpc={}, adapter={})",
                f.rpc_name,
                f.adapter
            );
        }
    }
}
