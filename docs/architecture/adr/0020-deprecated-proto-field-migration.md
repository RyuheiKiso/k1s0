# ADR-0020: Deprecated proto フィールドの段階的移行計画

## ステータス

承認済み（H-06/H-11 監査対応: 2026-03-29 に移行完了目標時期を追記）

## コンテキスト

auth.proto をはじめ複数の proto ファイルで、string 型フィールドから enum 型フィールドへの移行が進行中である。
具体的には以下のファイルにおいて、旧フィールド（string 型）と新フィールド（enum 型）が共存している状態にある。

- `api/proto/k1s0/system/auth/v1/auth.proto` — event_type / result（string → AuditEventType / AuditResult）
- `api/proto/k1s0/system/workflow/v1/workflow.proto` — step_type（string → WorkflowStepType）
- `api/proto/k1s0/system/dlq/v1/dlq.proto` — status（string → DlqMessageStatus）
- `api/proto/k1s0/system/featureflag/v1/featureflag.proto` — change_type（string → ChangeType）
- `api/proto/k1s0/system/config/v1/config.proto` — change_type（string → ChangeType）
- `api/proto/k1s0/system/ratelimit/v1/ratelimit.proto` — algorithm（string → RateLimitAlgorithm）
- `api/proto/k1s0/system/notification/v1/notification.proto` — status / 各 timestamp（string → enum / Timestamp 型）
- `api/proto/k1s0/system/saga/v1/saga.proto` — status（string → SagaStatus）

移行途中で旧フィールドと新フィールドが共存しており、以下の問題が生じていた。

1. コンパイラ（protoc）が deprecated フィールドの使用を警告しない（`[deprecated = true]` アノテーションがないため）
2. クライアント開発者がどちらのフィールドを使うべきか不明確
3. 旧フィールドへの誤ったデータ入力が継続するリスク

外部監査（A-4）でもこの問題が指摘された。

## 決定

deprecated proto フィールドの段階的移行を以下の 3 フェーズで実施する。

### Phase 1（即時）: アノテーション追加によるコンパイラ警告の有効化

全 deprecated フィールドに `[deprecated = true]` オプションを追加する。
これにより protoc が生成する各言語コードに deprecated マーカーが付与され、
クライアントコンパイル時に警告が発生するようになる。

対象フィールドのコメントには「Deprecated: XXX を使用すること」の記述が既に存在するが、
`[deprecated = true]` アノテーションが欠落していたため、本 ADR の対応として追加した。

### Phase 2（3 ヶ月以内）: 新フィールドへの移行完了

**完了目標**: 2026-06-30

<!-- H-06/H-11 監査対応: 移行完了目標時期を明記 -->

対象フィールドおよび移行先:

| サービス | 旧フィールド（string 型） | 新フィールド（enum 型） | 完了目標 |
|----------|--------------------------|------------------------|----------|
| auth | `event_type` | `AuditEventType` | 2026-06-30 |
| auth | `result` | `AuditResult` | 2026-06-30 |
| graphql-gateway | `event_type`, `result` | `AuditEventType`, `AuditResult` | 2026-06-30 |
| workflow | `step_type` | `WorkflowStepType` | 2026-06-30 |
| dlq | `status` | `DlqMessageStatus` | 2026-06-30 |
| featureflag | `change_type` | `ChangeType` | 2026-06-30 |
| config | `change_type` | `ChangeType` | 2026-06-30 |
| ratelimit | `algorithm` | `RateLimitAlgorithm` | 2026-06-30 |
| notification | `status` / 各 timestamp | enum / Timestamp 型 | 2026-06-30 |
| saga | `status` | `SagaStatus` | 2026-06-30 |

実施内容:
- 全 gRPC クライアント（Go / Rust / TypeScript / Dart）において、新フィールド（enum 型 / Timestamp 型）を優先使用するよう実装を更新する
- 旧フィールドへのデータ入力を停止する（読み取り互換性は維持）
- サーバー側実装において、旧フィールドが送信された場合は新フィールドへ自動変換する処理を実装する

### Phase 3（6 ヶ月以内）: 旧フィールドの削除

**完了目標**: 2026-09-30

<!-- H-06/H-11 監査対応: 移行完了目標時期を明記 -->

- 旧フィールドを `reserved` に変更してフィールド番号を保護する
- フィールド名も `reserved` に追加して再利用を防ぐ
- 旧フィールドを参照しているコードが残っていないことを CI で確認してから削除する

## 理由

- **`[deprecated = true]` の即時追加**: protoc の標準機能を活用することで、各言語の生成コードに deprecation 情報が自動的に伝播する。追加コストが低く、効果が高い。
- **段階的移行（3 フェーズ）**: 一括削除ではなく段階的に移行することで、クライアント側の対応時間を確保しつつ後方互換性を維持する。
- **reserved による保護**: Phase 3 で単純削除ではなく `reserved` を使うことで、将来的なフィールド番号の再利用によるデシリアライズ破損を防ぐ。

## 影響

**ポジティブな影響**:

- protoc 生成コードに deprecated マーカーが付与され、クライアント開発者が使用すべきフィールドを明確に把握できる
- 型安全性が向上し、enum による値の制約が強制される
- 監査指摘（A-4）への対応が完了する

**ネガティブな影響・トレードオフ**:

- Phase 2 で全クライアントの実装変更が必要となり、一時的に開発工数が発生する
- Phase 1 完了後、既存クライアントコードのビルドで deprecated 警告が発生し始める
- 旧フィールドが削除される Phase 3 までの間、proto 定義が冗長になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: 一括削除 | deprecated フィールドを即座に削除する | 既存クライアントの後方互換性が失われ、サービス障害のリスクがある |
| 案 B: アノテーションのみ追加・削除しない | `[deprecated = true]` を追加するが Phase 3 を実施しない | フィールドが永続的に残り proto 定義が肥大化する。将来的なメンテナンスコストが増大する |
| 案 C: フィールド番号の再利用 | 旧フィールドを削除して同番号で新型を定義 | proto3 の仕様上、フィールド番号の型変更はデシリアライズ破損を引き起こすため不可 |

## 参考

- [ADR-0006: proto バージョニング戦略](./0006-proto-versioning.md)
- [ADR-0004: Timestamp 型移行計画](./0004-timestamp-migration.md)
- [Protocol Buffers Language Guide: Field Options](https://protobuf.dev/programming-guides/proto3/#options)
- [Deprecating a Field (proto3 best practices)](https://protobuf.dev/best-practices/dos-donts/#deprecate-fields)
