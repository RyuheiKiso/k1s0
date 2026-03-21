# ADR-0013: Rust edition を 2021 に統一する

## ステータス

承認済み

## コンテキスト

k1s0 プロジェクトの Rust クレート群は、ほとんどが `edition = "2021"` を使用していた。
しかし以下の bb-* クレート 5 件は `edition = "2024"` を使用しており、
各 Cargo.toml に `# TODO: edition統一` コメントが残ったまま方針が未確定だった。

対象クレート:
- `regions/system/library/rust/bb-binding/Cargo.toml`
- `regions/system/library/rust/bb-core/Cargo.toml`
- `regions/system/library/rust/bb-pubsub/Cargo.toml`
- `regions/system/library/rust/bb-secretstore/Cargo.toml`
- `regions/system/library/rust/bb-statestore/Cargo.toml`

edition 2024 は Rust 1.85 以降で安定化された比較的新しいエディションであり、
let チェーン（`if let A && let B`）などの構文が使用可能になる。
bb-statestore の `memory.rs` では実際に let チェーン構文が使用されていた。

## 決定

プロジェクト全体の Rust edition を `"2021"` に統一する。
bb-* 5 クレートの `edition = "2024"` を `edition = "2021"` に変更し、
edition 2024 固有の構文（let チェーン等）は edition 2021 互換の書き方に書き換える。

## 理由

- **一貫性**: プロジェクト全体で同一の edition を使用することで、開発者が各クレートごとに
  言語仕様の差異を意識する必要がなくなる。
- **コードレビューの認知コスト低減**: edition ごとに異なる構文ルールを覚える必要がなく、
  すべてのクレートで同一の構文ルールが適用される。
- **安定性**: edition 2021 はすでに広く使われており、ツールチェーン・エコシステムの
  サポートが十分に安定している。
- **TODO 解消**: 長期間未解決だった TODO コメントを確定した方針で解消する。

## 影響

**ポジティブな影響**:

- プロジェクト全体の edition が統一され、保守性・可読性が向上する。
- TODO コメントが削除され、技術的負債が解消される。
- 新規クレート追加時に edition を迷わず `"2021"` に設定できる。

**ネガティブな影響・トレードオフ**:

- edition 2024 固有の構文（let チェーン、クロージャキャプチャの変更等）は使用不可になる。
- bb-statestore の `memory.rs` に記述されていた let チェーン構文をネストした `if let` に書き換えた（軽微な可読性低下）。

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: edition 2024 に全体統一 | 全クレートを edition 2024 に揃える | 大多数が edition 2021 のため影響範囲が大きく、ツールチェーン・依存ライブラリの対応状況を慎重に確認する必要がある。将来の移行タスクとして別途計画すべきと判断した。 |
| 案 B: 現状維持（混在を許容） | クレートごとに edition を自由に選択する | 一貫性がなくなりコードレビューコストが増大する。TODO も解消されず技術的負債が積み重なる。 |

## 参考

- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- [What's new in Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [What's new in Rust 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
