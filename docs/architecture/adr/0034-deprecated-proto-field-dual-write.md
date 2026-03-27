# ADR-0034: deprecated proto フィールド dual-write 移行戦略

## ステータス

採用済み（2026-03-26 更新: M-8 対応 — 移行マイルストーン設定）

## 背景

k1s0 プロジェクトでは、Protocol Buffers のフィールド型移行において複数の deprecated フィールドが存在する。
具体的には、文字列で表現されていたステータスや日時フィールドを、型安全な enum や Timestamp 型に移行する作業が進行中である。

移行対象のフィールドパターン:
- **文字列 → enum**: `event_type: string` → `event_type_enum: AuditEventType`
- **文字列 → enum**: `result: string` → `result_enum: AuditResult`
- **文字列 → Timestamp**: `created_at: string` → `created_at_ts: Timestamp`
- **文字列 → enum**: `status: string` → `status_enum: NotificationStatus`
- **文字列 → enum**: `step_type: string` → `step_type_enum: WorkflowStepType`
- **文字列 → enum**: `algorithm: string` → `algorithm_enum: RateLimitAlgorithm`
- **文字列 → enum**: `status: string` → `status_enum: SagaStatus`

影響サーバー: auth, graphql-gateway, notification, workflow, ratelimit, saga

## 決定内容

移行期間中は **dual-write パターン** を採用する。
新旧両方のフィールドに値を設定し、クライアントが段階的に新フィールドへ移行できるようにする。

```rust
// dual-write の例 (auth/audit_grpc.rs)
fn domain_audit_log_to_proto(log: &AuditLog) -> ProtoAuditLog {
    ProtoAuditLog {
        // 旧フィールド（deprecated）: 後方互換のため引き続き設定
        event_type: log.event_type.clone(),
        result: log.result.clone(),
        // 新フィールド: 型安全な enum 値を設定
        event_type_enum: event_type_str_to_enum(&log.event_type),
        result_enum: result_str_to_enum(&log.result),
        // ...
    }
}
```

## 理由

1. **後方互換性の維持**: 旧フィールドを参照しているクライアントへの影響をゼロにする
2. **段階的移行**: 各クライアントが独自のペースで新フィールドへ移行できる
3. **ロールバック安全性**: 問題が発生した場合、クライアントが旧フィールドにフォールバック可能
4. **gRPC の deprecation 仕様準拠**: proto の `[deprecated=true]` アノテーションに従い、旧フィールドも並行して維持

## 移行完了条件

以下の条件が揃った時点で旧フィールドの削除を検討する:
1. 全クライアント（React, Flutter, GraphQL Gateway）が新フィールドを参照するように更新された
2. 少なくとも1リリースサイクル（1ヶ月）以上 dual-write が安定稼働した
3. 旧フィールドへのアクセスがモニタリングで確認されなくなった

## 影響

- **サーバー側**: enum 変換ヘルパー関数の追加（コードの複雑性がわずかに増加）
- **クライアント側**: 変更なし（移行完了まで旧フィールドを引き続き使用可能）
- **Wire サイズ**: 両フィールドが送信されるため、一時的に proto メッセージサイズが増加する

## 代替案

- **一括置換**: 全クライアントを同時に移行する。リリース調整コストが高く、デプロイ順序に依存するリスクがあるため却下
- **バージョニング**: proto の新バージョンを別サービスとして提供する。運用コストが高すぎるため却下

## 移行マイルストーン（M-8 監査対応）

**更新日:** 2026-03-26

外部技術監査（M-8）の指摘を受け、deprecated フィールドの完全移行期限を設定する。

| マイルストーン | 期限 | 内容 |
|--------------|------|------|
| Phase 1 完了 | 2026-Q2（2026-06-30） | 全クライアント（React system-client, Flutter）が新フィールド（enum 版）を参照するように更新 |
| Phase 2 完了 | 2026-Q3（2026-09-30） | モニタリングで旧フィールドへのアクセスがゼロになったことを確認 |
| Phase 3 完了 | 2026-Q4（2026-12-31） | 旧フィールド（`event_type`, `result` 等の string 版）を proto から削除 |

**移行状況（2026-03-26 時点）:**

| フィールド | サーバー dual-write | クライアント移行 |
|-----------|-------------------|----------------|
| `event_type` → `event_type_enum` | 完了 | 未実施 |
| `result` → `result_enum` | 完了 | 未実施 |
| `created_at` → `created_at_ts` | 完了 | 未実施 |
| `status` (notification) → `status_enum` | 完了 | 未実施 |
| `step_type` → `step_type_enum` | 完了 | 未実施 |
| `algorithm` → `algorithm_enum` | 完了 | 未実施 |
| `status` (saga) → `status_enum` | 完了 | 未実施 |

## 参考資料

- [Protocol Buffers Field Deprecation](https://protobuf.dev/programming-guides/proto3/#options)
- 影響ファイル:
  - `regions/system/server/rust/auth/src/adapter/grpc/audit_grpc.rs`
  - `regions/system/server/rust/graphql-gateway/src/infrastructure/grpc/auth_client.rs`
  - `regions/system/server/rust/notification/src/adapter/grpc/tonic_service.rs`
  - `regions/system/server/rust/workflow/src/adapter/grpc/tonic_service.rs`
  - `regions/system/server/rust/ratelimit/src/adapter/grpc/tonic_service.rs`
  - `regions/system/server/rust/saga/src/adapter/grpc/tonic_service.rs`
