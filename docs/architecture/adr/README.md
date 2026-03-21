# Architecture Decision Records（ADR）

アーキテクチャ上の重要な決定事項を記録する。各 ADR は決定の背景・理由・影響・代替案を含み、なぜ現在の設計になっているかを説明する。

## ADR 一覧

| 番号 | タイトル | ステータス |
|------|---------|----------|
| [ADR-0001](./0001-template.md) | テンプレート | — |
| [ADR-0002](./0002-monorepo.md) | モノリポ採用 | 承認済み |
| [ADR-0003](./0003-four-languages.md) | 4 言語（Go / Rust / TypeScript / Dart）採用 | 承認済み |
| [ADR-0004](./0004-timestamp-migration.md) | カスタム Timestamp 型から google.protobuf.Timestamp への移行計画 | 提案 |
| [ADR-0005](./0005-error-response-format.md) | エラーレスポンス体系の統一 | 承認済み |
| [ADR-0006](./0006-proto-versioning.md) | Protobuf バージョニング戦略 | 承認済み |

## ADR の追加方法

1. `0001-template.md` をコピーして連番でファイルを作成する
2. ステータスを「提案」として PR を作成する
3. レビュー後に「承認済み」または「却下」に変更する

## ステータス定義

| ステータス | 意味 |
|----------|------|
| 提案 | レビュー中の決定案 |
| 承認済み | チームで合意した決定 |
| 却下 | 検討したが採用しなかった決定 |
| 廃止 | かつて有効だったが後続の ADR で置き換えられた決定 |

## 関連ドキュメント

- [アーキテクチャ概要](../overview/) — Tier 構成・全体設計方針
- [コーディング規約](../conventions/コーディング規約.md) — 命名規則・Linter 設定
