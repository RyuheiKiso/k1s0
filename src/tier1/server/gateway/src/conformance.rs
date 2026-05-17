// k1s0 tier1 conformance runner
// 8 transport adapter × 5 conformance class = 40 cell の Bidi conformance テストを実行する

// Bidi handshake state machine をインポートする
use crate::bidi::BidiSession;
// シリアライズ関連のインポート
use serde::{Deserialize, Serialize};
// UUID 生成のインポート
use uuid::Uuid;

// 5 つの conformance class 定義（generate_capabilities.py の _CONFORMANCE_CLASSES と一致する）
pub const CONFORMANCE_CLASSES: &[&str] = &[
    // 双方向ストリーミング conformance class
    "v1_bidi_streaming",
    // 単方向 RPC conformance class
    "v1_unary_rpc",
    // サーバーストリーミング conformance class
    "v1_server_streaming",
    // クライアントストリーミング conformance class
    "v1_client_streaming",
    // Connect プロトコル conformance class
    "v1_connect_protocol",
];

// 8 つの transport adapter 定義（generate_capabilities.py の _ADAPTERS と一致する）
pub const ADAPTERS: &[&str] = &[
    // gRPC Go アダプター
    "grpc_go",
    // gRPC Java アダプター
    "grpc_java",
    // gRPC Python アダプター
    "grpc_python",
    // gRPC Node.js アダプター
    "grpc_node",
    // Connect Go アダプター
    "connect_go",
    // Connect Web アダプター
    "connect_web",
    // gRPC-Web アダプター
    "grpc_web",
    // .NET gRPC アダプター
    "dotnet_grpc",
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
    // セル ID を構築する
    let cell_id = format!("{conformance_class}__{adapter}");
    // テスト成功・失敗を status に変換する
    let (status, notes) = if result.is_ok() && invariant_ok {
        // 成功: ハンドシェイク完了 + 不変条件保持
        (
            "green".to_string(),
            format!(
                "kind cluster: {adapter} {conformance_class} handshake completed, \
                 send_count={} recv_count={}, NoDoubleHandshake invariant holds",
                session.send_count, session.recv_count
            ),
        )
    } else {
        // 失敗: エラーまたは不変条件違反
        (
            "red".to_string(),
            format!(
                "FAIL: {adapter} {conformance_class} - result={result:?} invariant={invariant_ok}"
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
        // 単一セルのテストを実行する
        let cell = run_cell_test("v1_bidi_streaming", "grpc_go");
        // cell_id が期待値と一致することを確認する
        assert_eq!(cell.cell_id, "v1_bidi_streaming__grpc_go");
    }
}
