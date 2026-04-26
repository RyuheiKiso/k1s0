# `examples/tier1-rust-service/` — Rust 自作領域の最小例

tier1 の Rust 自作領域（ZEN Engine 統合 / 暗号 / 雛形 CLI / JTC 固有機能）の
スタイルを示す最小サービス例。

## 目的

- `src/tier1/rust/crates/` 配下のクレートと同じ構造（Edition 2024 / `#![forbid(unsafe_code)]` /
  panic は FFI 境界で `catch_unwind`）を新規メンバーが真似できる
- ZEN Engine ベースの decision-as-data の典型呼び出しパタンを示す
- E2E テストで「example が動くこと」を Golden Path の契約とする（CI 週次実行）

## 想定読者

- tier1 Rust コアの新規コミッタ
- 既存業務システムから Rust 領域へ拡張する開発者
- ZEN Engine による decision 表評価の実装パタンを学びたい人

## scope（リリース時点）

リリース時点では以下 3 点のみを満たす最小骨格を配置する:

1. Cargo workspace に組み込まれる `Cargo.toml` の例
2. `#![forbid(unsafe_code)]` を付けた `src/lib.rs` / `src/main.rs`
3. 最小の health check ロジック（panic-free / 型のみ）

**未実装（採用初期に拡張予定）:**

- ZEN Engine 統合の最小例
- `k1s0-sdk-proto` crate からの型 import 例
- `tonic` クライアントによる tier1 gRPC 呼び出し
- integration test（Testcontainers ベース、本物の OpenBao / Postgres を立てる）
- Dockerfile（distroless / nonroot / multi-stage）
- catalog-info.yaml（Backstage カタログ化）

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/04_rust_workspace配置.md`
- ADR-TIER1-001（Go + Rust ハイブリッド tier1）
- ADR-RULE-001（ZEN Engine）

## 起動方法（採用初期完成後の想定）

```bash
cd examples/tier1-rust-service
cargo run --release
```

## 参照する tier1 API

- DecisionService（ZEN Engine による業務ルール評価）
- AuditService（決定の入出力をハッシュチェーンに記録）
