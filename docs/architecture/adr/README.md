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
| [ADR-0025](./0025-terraform-state-s3.md) | Terraform State Backend を Ceph RGW（S3互換）に移行 | 承認済み |
| [ADR-0026](./0026-service-tier-db-integration.md) | Service Tier DB統合設計（board/activity を k1s0_service に統合） | 承認済み |
| [ADR-0027](./0027-db-app-user-role-separation.md) | DBアプリケーションユーザー権限分離（k1s0 非特権ロール） | 承認済み |
| [ADR-0028](./0028-tenant-id-acquisition.md) | マルチテナントID取得方式（JWT カスタムクレーム + gRPC メタデータ） | 承認済み |
| [ADR-0029](./0029-postgresql-ha-strategy.md) | PostgreSQL HA 戦略 | 承認済み |
| [ADR-0030](./0030-tier-access-dynamic-mapping.md) | tier_access クレームのロールベース動的マッピングへの移行 | 承認済み |
| [ADR-0031](./0031-etcd-encryption-at-rest.md) | etcd encryption-at-rest（保存時暗号化）の採用 | 承認済み |
| [ADR-0032](./0032-zod-v4-record-key-schema.md) | zod v4 API移行方針 — record キースキーマ必須化 | 承認済み |
| [ADR-0033](./0033-riverpod-v3-family-async-notifier.md) | Riverpod v3 FamilyNotifier → FamilyAsyncNotifier 移行 | 承認済み |
| [ADR-0034](./0034-deprecated-proto-field-dual-write.md) | deprecated proto フィールド dual-write 移行戦略 | 承認済み |
| [ADR-0035](./0035-dockerfile-template-strategy.md) | Dockerfile テンプレート戦略（27 サービス個別 Dockerfile の維持方針） | 承認済み |
| [ADR-0036](./0036-promtail-log-collection-strategy.md) | Promtail ログ収集戦略 | 承認済み |
| [ADR-0037](./0037-yaml-config-library-migration.md) | YAML 設定ライブラリ移行 | 提案中 |
| [ADR-0038](./0038-k8s-rbac-namespace-scoping.md) | Kubernetes RBAC 権限の Namespace スコープ化（ClusterRoleBinding → RoleBinding） | 承認済み |
| [ADR-0039](./0039-service-catalog-rest-client.md) | graphql-gateway の service-catalog クライアントを gRPC から REST へ変更 | 承認済み |
| [ADR-0040](./0040-grpc-port-range-hyper-v-avoidance.md) | gRPC ホストポートを Hyper-V 動的除外範囲外に移動 | 承認済み |
| [ADR-0041](./0041-ratelimit-api-path-alignment.md) | レートリミット API パスのクライアント・サーバー間統一 | 承認済み |
| [ADR-0042](./0042-bff-proxy-upstream-strategy.md) | BFF-Proxy upstream 拡張戦略 | 承認済み |
| [ADR-0043](./0043-service-tier-graphql-integration.md) | Service Tier GraphQL 統合方針 | 承認済み |
| [ADR-0044](./0044-file-event-topic-strategy.md) | file-rust Kafka イベントトピック設計方針 | 承認済み |
| [ADR-0045](./0045-vault-per-service-roles.md) | Vault サービス個別 Kubernetes Auth ロール実装計画 | 提案中 |
| [ADR-0046](./0046-tauri-gui-deferral.md) | Tauri GUI 機能の段階的延期戦略 | 承認済み |
| [ADR-0047](./0047-vault-secret-path-auth-server.md) | Vault シークレットパス設計（auth-server） | 承認済み |
| [ADR-0048](./0048-ratelimit-fail-closed-default.md) | レートリミット Fail-Closed デフォルト設計 | 承認済み |
| [ADR-0049](./0049-jwt-aud-vec-string.md) | JWT audience を Vec<String> で管理する設計 | 承認済み |
| [ADR-0050](./0050-advisory-lock-timeout-strategy.md) | pg_try_advisory_lock + リトライによる DB マイグレーション排他制御 | 承認済み |
| [ADR-0051](./0051-k8s-placeholder-ci-validation.md) | Kubernetes マニフェストのプレースホルダー自動検証 CI パイプライン設計 | 承認済み |

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

## ADR レビュープロセス（L-02 監査対応）

### レビュー頻度
- **四半期ごと**（1月・4月・7月・10月）に全 ADR をレビューする
- 新しい技術決定や設計変更があった場合は、関連 ADR を即時更新する

### レビュー観点
1. ステータスが「提案中」のまま放置されていないか
2. 実装済みの内容と ADR の記述が乖離していないか
3. TODO が残存しているまま陳腐化していないか
4. 技術スタックのバージョンアップが必要な ADR がないか

### ステータスライフサイクル
| ステータス | 説明 |
|-----------|------|
| 提案中 | 検討中。まだ承認されていない |
| 承認済み | 実装方針として決定した |
| 廃止済み | 別の ADR に置き換えられた |
| 非推奨 | 将来的に廃止予定 |

### 陳腐化した ADR の扱い
- ステータスを「廃止済み」に更新し、後継 ADR へのリンクを追加する
- 削除はしない（決定の歴史を保持するため）

## 関連ドキュメント

- [アーキテクチャ概要](../overview/) — Tier 構成・全体設計方針
- [コーディング規約](../conventions/コーディング規約.md) — 命名規則・Linter 設定
