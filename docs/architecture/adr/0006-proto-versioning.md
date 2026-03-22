# ADR-0006: Protobuf バージョニング戦略

## ステータス

承認済み

## コンテキスト

k1s0 は4言語（Go/Rust/TypeScript/Dart）でサービスを実装しており、
サービス間通信に Protobuf / gRPC を使用している。
技術品質監査（2026-03-21）の指摘（M-007）により、Proto のバージョニング戦略が
明文化されていないという問題が指摘された。

具体的な問題:
- フィールド削除時の `reserved` 使用ルールが統一されていない
- deprecated フィールドの廃止タイムラインが管理されていない
- 後方互換性のある変更と破壊的変更の区別が曖昧だった

## 決定

以下のバージョニング戦略を採用する:

### パッケージバージョニング

- パッケージ名に `v{major}` を含める: `k1s0.{tier}.{domain}.v{major}`
- 後方互換性を破壊する変更が必要な場合のみ `v2` パッケージを新設する
- `v1` と `v2` は独立して共存でき、クライアントは段階的に移行できる

### フィールドバージョニング（後方互換変更）

後方互換性のある変更（フィールド追加等）は同一パッケージで行う:

```protobuf
message Task {
  string id = 1;
  // Deprecated: status_enum を使用すること。
  string status = 3 [deprecated = true];
  // 新フィールド（既存クライアントは無視する）
  TaskStatus status_enum = 13;
}
```

### フィールド削除のルール（M-020）

フィールドを削除する場合は必ず `reserved` で番号を予約する:

1. deprecated フィールドに `[deprecated = true]` を付与する
2. 全クライアントの移行完了を確認する（最低1リリースサイクル待機）
3. フィールドを削除し、`reserved` で番号と名前を予約する:

```protobuf
message Task {
  reserved 3;
  reserved "status";
  TaskStatus status_enum = 13;
}
```

### CI による自動検証

`buf breaking` を CI で実行し、意図しない破壊的変更を自動検出する:

```yaml
- name: Check breaking changes
  run: buf breaking api/proto --against '.git#branch=main'
```

## 理由

- パッケージ名にバージョンを含めることで、既存クライアントへの影響を完全にゼロにできる
- `reserved` による予約でフィールド番号の再利用を防ぎ、Wire format の混乱を防止する
- CI の自動検証により、人的ミスによる破壊的変更を事前にブロックできる

## 影響

**ポジティブな影響**:

- フィールド番号の再利用による Wire format 破損を防止できる
- deprecated フィールドの管理が標準化され、廃止計画が明確になる
- CI による自動検証でレビュー負荷が軽減される

**ネガティブな影響・トレードオフ**:

- deprecated フィールドは移行期間中 Wire format 上に残るため、バイナリサイズが若干増加する
- `buf breaking` の制限により、意図的な破壊的変更は明示的な例外処理が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| URL バージョニング（/v1/orders） | REST API 流のバージョニング | gRPC パッケージ名との整合性が取れない |
| Schema Registry のみで管理 | Confluent Schema Registry の互換性チェックのみ | Proto の型安全性を活用できない |

## 参考

- [proto設計.md](../api/proto設計.md) — Proto 設計ガイドライン・バージョニング詳細
- [ADR-0004](./0004-timestamp-migration.md) — Timestamp 移行（deprecation の具体例）
- [buf 公式ドキュメント](https://buf.build/docs/breaking/overview)
