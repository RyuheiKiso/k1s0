# ADR-0004: カスタム Timestamp 型から google.protobuf.Timestamp への移行計画

## ステータス

承認済み（2026-03-26 更新: §5.7 対応 — Phase A 移行スケジュール設定）

## コンテキスト

k1s0 では `k1s0.system.common.v1.Timestamp` という独自の Timestamp 型を定義し、全サービスの Protobuf
定義で使用している。この型は `google.protobuf.Timestamp` と同一のフィールド定義（`seconds: int64`, `nanos: int32`）
を持つが、Well-Known Types に依存しないために独自型として定義された。

技術品質監査（2026-03-21）の指摘（M-007）により、以下の問題が明らかになった:

- Well-Known Types のエコシステム（JSON 変換、gRPC-Gateway、各言語の標準ライブラリ）が利用できない
- `google.protobuf.Timestamp` との相互変換コストが発生する
- `timestamp_to_datetime()` / `datetime_to_timestamp()` 等のヘルパー関数の維持コストがある

## 決定

`k1s0.system.common.v1.Timestamp` を段階的に廃止し、`google.protobuf.Timestamp` に移行する。
移行は3フェーズで行い、破壊的変更を最小化する。

**Phase A（準備）**: 既存フィールドに `[deprecated = true]` を付与し、ビルドを継続する。
**Phase B（段階移行）**: 新規フィールドから `google.protobuf.Timestamp` を使用する。
**Phase C（完了）**: 全フィールドを移行後、独自型を削除する。

## 理由

- `google.protobuf.Timestamp` は Protobuf エコシステムで標準的に扱われ、gRPC-Gateway・
  OpenAPI 生成・各言語の時刻ライブラリとの親和性が高い
- Well-Known Types は buf/protoc で自動解決され、追加の管理コストが不要
- 移行後は `datetime_to_timestamp()` 等のカスタムヘルパーが不要になり、コードが簡素化される

## 影響

**ポジティブな影響**:

- gRPC-Gateway による REST レスポンスの JSON 変換が標準化される
- 各言語クライアントで標準の時刻型（Go の `time.Time`、Rust の `chrono::DateTime`）との変換が簡素化される
- エコシステムツール（GraphQL スキーマ生成等）との互換性が向上する

**ネガティブな影響・トレードオフ**:

- 移行は破壊的変更（Wire format は同一だがパッケージ名が変わる）のため慎重な段階的移行が必要
- 全サービス・全クライアント（Flutter/TypeScript）の同時移行調整が必要
- `buf breaking` により意図的な変更として扱う必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 現状維持 | `k1s0.system.common.v1.Timestamp` を継続使用 | エコシステムとの乖離が継続し、技術負債が蓄積する |
| 即時置換 | 一括で全フィールドを移行 | 破壊的変更を一度に行うため、デプロイリスクが高い |

## 移行スケジュール（§5.7 対応）

**更新日:** 2026-03-26

外部技術監査（§5.7）の指摘を受け、Timestamp 移行の具体的なスケジュールを設定する。

### Phase A: 準備フェーズ（deprecated アノテーション付与）

| 期限 | 内容 | ステータス |
|------|------|-----------|
| 2026-Q2（2026-06-30） | 全 proto ファイルの `k1s0.system.common.v1.Timestamp` フィールドに `[deprecated = true]` を付与 | 未実施 |
| 2026-Q2（2026-06-30） | 新規フィールドとして `*_ts: google.protobuf.Timestamp` を並行追加（dual-write 開始） | 未実施 |

### Phase B: 段階移行フェーズ

| 期限 | 内容 | ステータス |
|------|------|-----------|
| 2026-Q3（2026-09-30） | サーバー実装を `google.protobuf.Timestamp` フィールドで読み書きするよう更新 | 未実施 |
| 2026-Q3（2026-09-30） | Go・Rust クライアントライブラリの Timestamp ヘルパー更新 | 未実施 |
| 2026-Q3（2026-09-30） | React/Flutter クライアントの移行完了 | 未実施 |

### Phase C: 完了フェーズ

| 期限 | 内容 | ステータス |
|------|------|-----------|
| 2026-Q4（2026-12-31） | `k1s0.system.common.v1.Timestamp` フィールドを proto から削除 | 未実施 |
| 2026-Q4（2026-12-31） | `timestamp_to_datetime()` 等のカスタムヘルパー関数を削除 | 未実施 |

**前提条件:** ADR-0034 の deprecated フィールド移行（M-8）の Phase 1 完了後に Phase B を開始する。

## 参考

- [proto設計.md](../api/proto設計.md) — Proto設計ガイドライン
- [ADR-0003](./0003-four-languages.md) — 4言語採用（TypeScript/Dart クライアントへの影響あり）
- [Protobuf Well-Known Types](https://protobuf.dev/reference/protobuf/google.protobuf/#timestamp)
