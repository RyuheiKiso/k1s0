// 本ファイルは tier1 Rust 自作領域 Golden Path 最小例のライブラリエントリ。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
//       docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/04_rust_workspace配置.md
// 関連 ID: ADR-TIER1-001（Go + Rust ハイブリッド）/ ADR-RULE-001（ZEN Engine）
//
// 役割:
//   tier1 Rust 自作領域の「最小だが本番形」を示す。本クレートは
//     - `#![forbid(unsafe_code)]` を crate root に配置
//     - panic-free を保証する型のみの API
//   までを 1 ファイルで読み切れる範囲に収める。
//
// scope（リリース時点）:
//   - HealthStatus 型と check 関数のみ
//
// 採用初期で拡張:
//   - tonic クライアントによる tier1 gRPC 呼び出し
//   - ZEN Engine 評価エンジンの統合
//   - integration test（Testcontainers）

// crate 全域で unsafe を禁止する（tier1 Rust 規約）。
#![forbid(unsafe_code)]
// 全 public 項目に doc コメント必須化（採用初期で本格 API 追加時の品質ゲート）。
#![warn(missing_docs)]

//! k1s0 Golden Path: tier1 Rust 自作領域の最小例ライブラリ。
//!
//! `tier1-rust-service` は ZEN Engine 統合や暗号処理等を行う Rust コアの
//! 「最小だが本番形」のテンプレートとして配置される。リリース時点では
//! ヘルスチェックのみ提供し、採用初期に gRPC / ZEN を追加していく前提。

/// サービスのヘルスチェック結果。
///
/// gRPC 標準 health protocol（grpc.health.v1.Health）の `Serving` / `NotServing` /
/// `Unknown` に 1:1 で対応する。tier1 facade との型整合のため独立 enum で表現する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// 正常に応答可能。
    Serving,
    /// 一時的に応答不可（依存サービス切断 / 起動中等）。
    NotServing,
    /// ヘルスチェック自体が判定不能。
    Unknown,
}

/// 現在のヘルスチェック結果を返す。
///
/// リリース時点では常に `Serving` を返す。採用初期で
/// 依存（Postgres / OpenBao / ZEN Engine）の到達性を実装する。
#[must_use]
pub fn check() -> HealthStatus {
    // 最小骨格: 何の依存も持たないため常に Serving を返す。
    HealthStatus::Serving
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_default_serving() {
        // 初期状態で Serving を返すことを保証する（採用初期で依存追加後も
        // mock 経由で当該テストが意味を保てるよう、入力なしの形で固定する）。
        assert_eq!(check(), HealthStatus::Serving);
    }
}
