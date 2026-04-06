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
| [ADR-0045](./0045-vault-per-service-roles.md) | Vault サービス個別 Kubernetes Auth ロール実装計画 | 実装完了 |
| [ADR-0046](./0046-tauri-gui-deferral.md) | Tauri GUI 機能の段階的延期戦略 | 承認済み |
| [ADR-0047](./0047-vault-secret-path-auth-server.md) | Vault シークレットパス設計（auth-server） | 承認済み |
| [ADR-0048](./0048-ratelimit-fail-closed-default.md) | レートリミット Fail-Closed デフォルト設計 | 承認済み |
| [ADR-0049](./0049-jwt-aud-vec-string.md) | JWT audience を Vec<String> で管理する設計 | 承認済み |
| [ADR-0050](./0050-advisory-lock-timeout-strategy.md) | pg_try_advisory_lock + リトライによる DB マイグレーション排他制御 | 承認済み |
| [ADR-0051](./0051-k8s-placeholder-ci-validation.md) | Kubernetes マニフェストのプレースホルダー自動検証 CI パイプライン設計 | 承認済み |
| [ADR-0052](./0052-jsonb-column-encryption.md) | JSONB カラム暗号化戦略 | 部分実装済み |
| [ADR-0053](./0053-auth-config-nested-format.md) | AuthConfig ネスト形式統一 | 承認済み |
| [ADR-0054](./0054-rls-remaining-tenant-tables.md) | RLS 段階的実装戦略 | 承認済み |
| [ADR-0055](./0055-audit-response-2026-03-29.md) | 2026-03-29 外部技術監査対応 | 承認済み |
| [ADR-0056](./0056-multitenancy-scope-decision.md) | マルチテナント設計の明確化 | 承認済み |
| [ADR-0057](./0057-kong-jwt-istio-authz-separation.md) | Kong JWT/Istio AuthZ 分離 | 承認済み |
| [ADR-0058](./0058-graphql-record-audit-log-input-breaking-change.md) | GraphQL RecordAuditLogInput 破壊的変更 | 承認済み |
| [ADR-0059](./0059-grafana-postgresql-migration.md) | Grafana SQLite → PostgreSQL 移行 | 承認済み |
| [ADR-0060](./0060-saga-dedicated-database.md) | saga-rust 専用データベース分離 | 承認済み |
| [ADR-0061](./0061-ropc-to-client-credentials-migration.md) | Keycloak ROPC から Client Credentials Grant への移行 | 承認済み（STATIC-MEDIUM-003 監査対応）|
| [ADR-0062](./0062-distributed-lock-connection-cleanup.md) | distributed-lock PostgreSQL 接続クリーンアップ戦略 | 承認済み |
| [ADR-0063](./0063-aes-gcm-aad-session-binding.md) | セッション暗号化 AAD 導入 | 承認済み |
| [ADR-0064](./0064-tenant-isolation-db-and-cache.md) | マルチテナント分離 DB+キャッシュ | 承認済み |
| [ADR-0065](./0065-cosign-signature-verification.md) | app-registry Cosign 署名検証 | 承認済み |
| [ADR-0066](./0066-config-value-encryption.md) | config サービス設定値 AES-256-GCM 暗号化 | 承認済み |
| [ADR-0067](./0067-bff-proxy-ssrf-allowlist.md) | BFF プロキシ SSRF ホワイトリスト | 承認済み |
| [ADR-0068](./0068-readyz-response-format.md) | readyz エンドポイントレスポンス形式の標準化 | 承認済み |
| [ADR-0069](./0069-audit-response-2026-04-02.md) | 外部技術監査 2026-04-02 対応記録 | 承認済み |
| [ADR-0070](./0070-db-isolation-event-monitor-master-maintenance.md) | event-monitor / master-maintenance の独立 DB 分離 | 承認済み |
| [ADR-0071](./0071-tonic-012-proto-compatibility.md) | tonic 0.12 互換のための Proto 生成コード対応方針 | 承認済み |
| [ADR-0072](./0072-rls-policy-type-cast-standardization.md) | RLS ポリシーの tenant_id 型キャスト標準化 | 承認済み |
| [ADR-0073](./0073-vault-terraform-yaml-sync-strategy.md) | Vault Terraform/YAML 同期戦略 | 承認済み |
| [ADR-0074](./0074-k1s0-common-template-validation-ci.md) | k1s0-common Helm テンプレート CI 検証 | 承認済み |
| [ADR-0075](./0075-vault-backup-dynamic-token.md) | Vault バックアップ動的トークン | 承認済み |
| [ADR-0076](./0076-gitops-networkpolicy-drift-prevention.md) | GitOps NetworkPolicy ドリフト防止 | 承認済み |
| [ADR-0077](./0077-vault-business-service-role-isolation.md) | Vault ビジネス/サービス層ロール分離 | 承認済み |
| [ADR-0078](./0078-audit-response-2026-04-03.md) | 外部技術監査 2026-04-03 対応記録（v1） | 承認済み |
| [ADR-0079](./0079-external-audit-remediation-2026-04-03-v2.md) | 外部技術監査 2026-04-03 v2 対応記録 | 承認済み |
| [ADR-0080](./0080-external-audit-remediation-2026-04-03-v3.md) | 外部技術監査対応 v3（2026-04-03） | 承認済み |
| [ADR-0081](./0081-external-audit-remediation-2026-04-04-v4.md) | 外部技術監査 v4 対応（2026-04-04） | 承認済み |
| [ADR-0082](./0082-graphql-pagination-unification.md) | GraphQL ページネーション統一方針 | 承認済み |
| [ADR-0083](./0083-task-status-transition-enforcement.md) | task ステータス遷移の検証レイヤー方針 | 承認済み |
| [ADR-0084](./0084-graphql-cursor-pagination-stability.md) | GraphQL カーソルページネーションの安定性 | 承認済み |
| [ADR-0085](./0085-library-language-parity.md) | ライブラリの言語パリティ方針（TypeScript/Dart の意図的非対称） | 承認済み |
| [ADR-0086](./0086-ansible-inventory-vault.md) | Ansible inventory の Vault 化ロードマップ | 承認済み（移行中） |
| [ADR-0087](./0087-cert-manager-external-pki.md) | cert-manager 外部 PKI 統合ロードマップ | 承認済み（移行計画中） |
| [ADR-0088](./0088-ansible-inventory-separation.md) | Ansible インベントリの環境分離方針 | 承認済み |
| [ADR-0089](./0089-vault-role-per-service-business-service-tier.md) | Vault ロール個別化の business/service tier 拡張 | 承認済み |
| [ADR-0090](./0090-aes-gcm-aad-introduction.md) | AES-GCM AAD（Additional Authenticated Data）導入 | 承認済み |
| [ADR-0091](./0091-jwt-token-introspection-hybrid.md) | JWT Token Introspection ハイブリッド方式 | 承認済み |
| [ADR-0092](./0092-aes-gcm-siv-migration-consideration.md) | AES-GCM-SIV 移行検討（M-002 監査対応） | 提案中 |
| [ADR-0093](./0093-tenant-id-uuid-to-text-migration.md) | tenant_id 型統一 — featureflag-db・config-db の UUID→TEXT マイグレーション | 承認済み |
| [ADR-0094](./0094-ts-auth-jwt-id-token-verification.md) | TypeScript Auth ライブラリ JWT id_token 署名検証の追加 | 承認済み |
| [ADR-0095](./0095-cli-template-version-alignment.md) | CLI テンプレートバージョンのワークスペース同期ポリシー | 承認済み |
| [ADR-0096](./0096-dart-auth-flutter-dependency-abstraction.md) | Dart Auth ライブラリの flutter_secure_storage 依存抽象化 | 承認済み |
| [ADR-0097](./0097-cross-service-fk-and-error-codes.md) | クロスサービス FK 不在の設計根拠と Proto エラーコード方針 | 承認済み |
| [ADR-0098](./0098-graphql-proto-alignment-policy.md) | GraphQL スキーマと Protocol Buffers の整合方針 | 承認済み |
| [ADR-0099](./0099-proto-field-type-migration-policy.md) | Protocol Buffers フィールド型移行方針（aud repeated・Timestamp 統一・reserved 宣言） | 承認済み |
| [ADR-0100](./0100-vault-audit-log-rotation.md) | Vault 監査ログローテーション管理方針 | 承認済み |
| [ADR-0101](./0101-graphql-pagination-relay-cursor.md) | GraphQL ページネーション統一方針（Relay Cursor ベース） | 承認済み |
| [ADR-0102](./0102-graphql-handler-split-deferred.md) | graphql_handler.rs の分割延期方針 | 承認済み |
| [ADR-0103](./0103-kafka-init-tmpfs-java-tmpdir.md) | kafka-init の /tmp:noexec 維持と JAVA_TOOL_OPTIONS による JVM 展開先変更 | 承認済み |
| [ADR-0104](./0104-aes-gcm-legacy-fallback-removal.md) | AES-GCM レガシーフォールバック削除 | 承認済み |
| [ADR-0105](./0105-registry-services-global-design.md) | レジストリサービスのシステムグローバル設計 | 承認済み |
| [ADR-0106](./0106-event-store-tenant-isolation.md) | event-store テナント分離実装 | 承認済み |
| [ADR-0107](./0107-s2s-jwt-rs256-migration.md) | サービス間 JWT の RS256 非対称鍵への移行 | 承認済み |
| [ADR-0108](./0108-vault-auth-terraform-single-source.md) | Vault 認証設定の Terraform 一元管理（ConfigMap 廃止） | 承認済み |
| [ADR-0109](./0109-vault-eventmonitor-maintenance-tenant-isolation.md) | vault-db / event-monitor-db / master-maintenance-db のテナント分離除外設計 | 承認済み |
| [ADR-0110](./0110-outbox-bypassrls-publisher-role.md) | outbox_events テーブルの BYPASSRLS ロール移行ロードマップ | 承認済み（実装は将来フェーズ） |
| [ADR-0111](./0111-flutter-certificate-pinning.md) | Flutter モバイルクライアントの証明書ピンニング実装方針 | 承認済み |
| [ADR-0112](./0112-external-audit-remediation-2026-04-06.md) | 外部技術監査対応（2026-04-06） | 承認済み |
| [ADR-0113](./0113-tauri-csp-unsafe-inline.md) | Tauri GUI の CSP に `unsafe-inline` を許可する | 承認済み |
| [ADR-0114](./0114-grpc-port-binding-strategy.md) | gRPC ポートバインド戦略（Windows Hyper-V 動的排除範囲への対応） | 承認済み |
| [ADR-0115](./0115-migration-execution-strategy.md) | DB マイグレーション実行戦略（外部実行 vs 起動時自動実行） | 承認済み |

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
