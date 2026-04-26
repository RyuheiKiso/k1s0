// 本ファイルは tier1 Rust 自作領域 Golden Path 最小例のバイナリエントリ。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
//       docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/04_rust_workspace配置.md
// 関連 ID: ADR-TIER1-001 / ADR-RULE-001
//
// 役割:
//   `cargo run --bin example-rust-service` で起動可能な最小バイナリ。
//   採用初期では tonic gRPC server に置換される予定だが、リリース時点では
//   依存ゼロで health check 結果を 1 行 stdout に出して exit する形を取る。

// crate root 規約: unsafe 禁止 / missing_docs warn は lib.rs 側で宣言済。
// バイナリ側は doc コメント必須化を緩める（test ハーネス・panic ハンドラ等が増えるため）。
#![forbid(unsafe_code)]

// lib.rs から型と関数を取り込む。
use k1s0_example_tier1_rust_service::{HealthStatus, check};

/// バイナリエントリポイント。
///
/// scope（リリース時点）: health check 結果を stdout に 1 行出して exit するだけ。
/// 採用初期で tonic gRPC server に置換される。
fn main() {
    // health check を呼び出して結果を取得する。
    let status = check();

    // 結果を識別可能な形で stdout に出力する。CI E2E test が grep で検証する想定。
    let label = match status {
        // 正常応答 → CI test の期待値。
        HealthStatus::Serving => "serving",
        // 起動中 / 依存切断時 → 採用初期で実装。
        HealthStatus::NotServing => "not_serving",
        // 判定不能 → 設計上は想定外パス。
        HealthStatus::Unknown => "unknown",
    };

    // 1 行 1 token で出力する（grep の容易性を優先）。
    println!("k1s0-example-tier1-rust-service: health={label}");
}
