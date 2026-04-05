# ADR-0099: Protocol Buffers フィールド型移行方針（aud repeated・Timestamp統一・reserved宣言）

## ステータス

承認済み

## コンテキスト

外部技術監査（2026-04-04）で Protocol Buffers 定義の型不整合が発見された。

**問題点（CRIT-006）:**

1. `auth.proto TokenClaims.aud`: `string` 型だが JWT 仕様（RFC 7519 Section 4.1.3）では配列型
   - 実際の Keycloak レスポンスは配列形式
   - Rust ドメイン層は `Vec<String>` で保持しているが gRPC 応答は先頭値のみ返していた

2. `file.proto FileMetadata.created_at/updated_at`: `string` 型（RFC 3339 文字列）だが他サービスは `k1s0.system.common.v1.Timestamp` を使用
   - タイムゾーン処理・パース効率・型安全性が劣る

3. `auth.proto` 複数メッセージの deprecated フィールド: `reserved` 宣言が TODO コメントとして保留中
   - フィールド番号の再利用リスクが潜在する

4. `navigation.proto`: `buf/validate/validate.proto` を import しているが未使用
   - buf lint 警告の原因

## 決定

### auth.proto TokenClaims.aud → repeated string

```protobuf
// Before: string aud = 3;
// After:
repeated string aud = 3;  // JWT spec 準拠（RFC 7519 Section 4.1.3）
```

- Go: `Aud []string`
- TypeScript: `aud: string[]`
- Rust: `pub aud: Vec<String>`
- Dart: `List<String> get aud`

ドメイン/ライブラリ層も合わせて更新:
- Rust auth library: `Claims.aud: Audience(Vec<String>)` → 変更なし（すでに正しい）
- Go auth library: `Claims.Audience []string` → 変更なし（すでに正しい）
- TypeScript: `Claims.aud: string | string[]` に変更（JWT 仕様準拠）
- Dart: `Claims.aud: List<String>` に変更

### file.proto FileMetadata Timestamp 統一

```protobuf
// Before:
// string created_at = 8;
// string updated_at = 9;
// After:
k1s0.system.common.v1.Timestamp created_at = 8;
k1s0.system.common.v1.Timestamp updated_at = 9;
```

ブレイキングチェンジだが、プロジェクトはまだベータ段階であり整合性を優先する。

### deprecated フィールドの reserved 宣言

以下のメッセージで deprecated フィールド削除期限（2026-06）まで TODO コメントを維持する:
- `auth.proto RecordAuditLogRequest` (field 1, 7)
- `auth.proto SearchAuditLogsRequest` (field 3, 6)
- `auth.proto AuditLog` (field 2, 8)
- `workflow.proto WorkflowStep` (field 3)
- `featureflag.proto WatchFeatureFlagResponse` (field 2)
- `config.proto WatchConfigResponse` (field 8)
- `saga.proto SagaStateProto` (field 4)

2026-06 に deprecated フィールドを削除と同時に reserved 宣言を有効化する。
削除前に全コンシューマーが enum フィールドへ移行していることを確認すること。

### navigation.proto 未使用 import 削除

`buf/validate/validate.proto` import を削除し、buf lint をクリーンに保つ。

## 理由

- JWT 仕様への準拠により、複数 audience を持つトークンが正しく処理される
- Timestamp 型統一により、タイムゾーン問題・パース負荷・型安全性が改善される
- reserved 宣言は実際のフィールド削除と同時に行わないとコンパイルエラーになる
- 段階的移行（deprecated → reserved）により後方互換を維持しながら移行を完了できる

## 影響

**ポジティブな影響:**

- JWT 複数 audience（Keycloak のデフォルト）が正しくパースされる
- ファイルタイムスタンプの精度・型安全性が向上する
- buf lint 警告が解消される

**ネガティブな影響・トレードオフ:**

- `aud` の型変更は wire format ブレイキングチェンジ（フィールド番号は同一でラベルが変わる）
- `FileMetadata.created_at/updated_at` の型変更は wire format ブレイキングチェンジ
- 既存の Dart/Go クライアントはアップデートが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| aud を string のまま | 先頭値のみを使用 | JWT 仕様違反・複数 audience を失う |
| 新フィールドとして追加 | `aud_list: repeated string` を追加 | フィールド重複で混乱を招く |
| google.protobuf.Timestamp | Well-Known Types を使用 | 追加依存を避けるため独自型を維持（Phase 1 完了後に移行予定） |
| string Timestamp のまま | RFC 3339 文字列を継続使用 | 型安全性・効率性が劣る |

## 参考

- RFC 7519 Section 4.1.3: Audience Claim
- [ADR-0022: buf validate 導入](0022-buf-validate-introduction.md)
- `api/proto/k1s0/system/auth/v1/auth.proto`
- `api/proto/k1s0/system/file/v1/file.proto`
- `api/proto/k1s0/system/common/v1/types.proto`

## HIGH-017: master_maintenance.proto Timestamp 完全移行（2026-04-04）

外部技術監査 HIGH-017 として `master_maintenance.proto` の以下6フィールドが `string` 型のまま残存していることが発覚した。

| メッセージ | フィールド | フィールド番号 |
|-----------|----------|-------------|
| `TableRelationship` | `created_at` | 8 |
| `ImportJob` | `started_at` | 10 |
| `ImportJob` | `completed_at` | 11 |
| `DisplayConfig` | `created_at` | 7 |
| `DisplayConfig` | `updated_at` | 8 |
| `AuditLogEntry` | `created_at` | 11 |

全6フィールドを `k1s0.system.common.v1.Timestamp` へ変更した。`ImportJob.completed_at` は未完了ジョブで null を返せるよう `optional k1s0.system.common.v1.Timestamp` とした。

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（CRIT-006 外部技術監査対応） | @k1s0-team |
| 2026-04-04 | HIGH-017 対応: master_maintenance.proto の string Timestamp 6フィールドを Timestamp 型へ移行 | @k1s0-team |
