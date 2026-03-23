# Architecture Decision Records（ADR）

アーキテクチャ上の重要な決定事項を記録する。各 ADR は決定の背景・理由・影響・代替案を含み、なぜ現在の設計になっているかを説明する。

## ADR 一覧

| 番号 | タイトル | ステータス |
|------|---------|----------|
| [ADR-0001](./0001-template.md) | テンプレート | — |
| [ADR-0002](./0002-monorepo.md) | モノリポ採用 | 承認済み |
| [ADR-0003](./0003-four-languages.md) | 4 言語（Go / Rust / TypeScript / Dart）採用 | 承認済み |
| [ADR-0004](./0004-timestamp-migration.md) | カスタム Timestamp 型から google.protobuf.Timestamp への移行計画 | 承認済み |
| [ADR-0005](./0005-error-response-format.md) | エラーレスポンス体系の統一 | 承認済み |
| [ADR-0006](./0006-proto-versioning.md) | Protobuf バージョニング戦略 | 承認済み |
| [ADR-0007](./0007-saga-compensation-inventory-reservations.md) | Saga 補償トランザクションのための inventory_reservations テーブル導入 | 承認済み |
| [ADR-0008](./0008-jwt-key-rotation.md) | JWT 秘密鍵ローテーション手順 | 承認済み |
| [ADR-0009](./0009-auth-navigation-boundary.md) | Auth サービスと Navigation サービスの責務境界 | 承認済み |
| [ADR-0010](./0010-idempotency-atomic-cas.md) | 冪等性と原子的CAS操作 | 承認済み |
| [ADR-0011](./0011-rbac-admin-privilege-separation.md) | RBAC管理者権限分離 | 承認済み |
| [ADR-0012](./0012-system-tier-rls-scope.md) | システム層RLSスコープ | 承認済み |
| [ADR-0013](./0013-rust-edition-standardization.md) | Rustエディション標準化 | 承認済み |
| [ADR-0014](./0014-auth-db-migration-roundtrip-fix.md) | 認証DBマイグレーション往復修正 | 承認済み |
| [ADR-0015](./0015-remove-s3-dependency.md) | S3依存除去 | 承認済み |
| [ADR-0016](./0016-kafka-kraft-migration.md) | Kafka KRaft移行 | 承認済み |
| [ADR-0017](./0017-kong-oidc-migration.md) | Kong OIDCプラグイン移行 | 承認済み |
| [ADR-0018](./0018-messaging-abstraction-unification.md) | メッセージング抽象化統一 | 承認済み |
| [ADR-0019](./0019-vault-domain-secret-isolation.md) | Vaultドメインシークレット分離 | 承認済み |
| [ADR-0020](./0020-deprecated-proto-field-migration.md) | 非推奨Protoフィールド移行 | 承認済み |
| [ADR-0021](./0021-cross-service-referential-integrity.md) | サービス間参照整合性 | 承認済み |
| [ADR-0022](./0022-grpc-message-validation-plan.md) | gRPCメッセージ検証計画 | 承認済み |
| [ADR-0023](./0023-helm-oci-registry.md) | Helm OCIレジストリ | 承認済み |
| [ADR-0024](./0024-service-account-isolation.md) | サービスアカウント分離 | 承認済み |
| [ADR-0025](./0025-terraform-state-s3.md) | TerraformステートS3 | 承認済み |
| [ADR-0026](./0026-service-tier-db-integration.md) | Service Tier DB統合設計（board/activity を k1s0_service に統合） | 承認済み |
| [ADR-0027](./0027-db-app-user-role-separation.md) | DBアプリケーションユーザー権限分離（k1s0 非特権ロール） | 承認済み |
| [ADR-0028](./0028-tenant-id-acquisition.md) | マルチテナントID取得方式（JWT カスタムクレーム + gRPC メタデータ） | 承認済み |

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
