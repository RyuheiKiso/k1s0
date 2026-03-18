# system-tenant-server 実装設計

> **注記**: 本ドキュメントは tenant-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-tenant-server（テナント管理サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（テナント CRUD・ライフサイクル管理・メンバー管理・プロビジョニング） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・Keycloak連携・Saga連携・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/tenant/)

### ディレクトリ構成

```
regions/system/server/rust/tenant/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── tenant.rs                                # Tenant エンティティ（TenantStatus 状態遷移）
│   │   │   ├── tenant_member.rs                         # TenantMember エンティティ
│   │   │   ├── provisioning.rs                          # ProvisioningJob エンティティ
│   │   │   └── pagination.rs                            # ページネーション共通型
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── tenant_repository.rs                     # TenantRepository トレイト
│   │   │   └── member_repository.rs                     # TenantMemberRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── tenant_domain_service.rs                 # テナント名重複チェック・ステータス遷移バリデーション
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_tenant.rs                             # テナント作成（Saga パターン・プロビジョニング開始）
│   │   ├── get_tenant.rs                                # テナント取得
│   │   ├── list_tenants.rs                              # テナント一覧
│   │   ├── update_tenant.rs                             # テナント更新
│   │   ├── suspend_tenant.rs                            # テナント停止
│   │   ├── activate_tenant.rs                           # テナント再開
│   │   ├── delete_tenant.rs                             # テナント論理削除
│   │   ├── add_member.rs                                # メンバー追加
│   │   ├── list_members.rs                              # メンバー一覧
│   │   ├── update_member_role.rs                        # メンバーロール更新
│   │   ├── remove_member.rs                             # メンバー削除
│   │   ├── get_provisioning_status.rs                   # プロビジョニング状態取得
│   │   └── watch_tenant.rs                              # テナント変更監視
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── tenant_handler.rs                        # axum REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── tenant_grpc.rs                           # gRPC サービス実装
│   │   │   ├── tonic_service.rs                         # tonic サービスラッパー
│   │   │   └── watch_stream.rs                          # テナント変更ストリーム
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                                  # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── tenant_postgres.rs                       # TenantRepository PostgreSQL 実装
│   │       └── member_postgres.rs                       # TenantMemberRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── keycloak_admin.rs                            # Keycloak Admin API クライアント（realm CRUD）
│   │   ├── saga_client.rs                               # Saga サーバー連携クライアント
│   │   ├── kafka_producer.rs                            # Kafka プロデューサー（テナントイベント配信）
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.tenant.v1.rs
│       └── k1s0.system.common.v1.rs
├── tests/
│   ├── integration_test.rs                              # 統合テスト
│   └── usecase_test.rs                                  # ユースケーステスト
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **TenantDomainService**: テナント名の一意性チェック、ステータス遷移バリデーション（`provisioning -> active -> suspended -> deleted`）を担当する。不正な遷移は `SYS_TENANT_INVALID_STATUS` で拒否する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateTenantUseCase` | テナント作成・Saga パターンによるプロビジョニング開始 |
| `GetTenantUseCase` / `ListTenantsUseCase` | テナント取得・一覧（ページネーション） |
| `UpdateTenantUseCase` | テナント表示名・プラン更新 |
| `SuspendTenantUseCase` / `ActivateTenantUseCase` | テナント停止・再開（ステータス遷移） |
| `DeleteTenantUseCase` | テナント論理削除（Saga パターンで Keycloak realm 無効化・DB スキーマアーカイブ） |
| `AddMemberUseCase` / `ListMembersUseCase` / `UpdateMemberRoleUseCase` / `RemoveMemberUseCase` | テナントメンバー管理 |
| `GetProvisioningStatusUseCase` | プロビジョニングジョブの状態取得 |
| `WatchTenantUseCase` | テナント変更のリアルタイム監視 |

#### 外部連携

- **Keycloak Admin** (`infrastructure/keycloak_admin.rs`): テナント作成時に Keycloak realm (`k1s0-{tenant_name}`) を自動作成・メンバー追加時にユーザー登録を行う
- **Saga Client** (`infrastructure/saga_client.rs`): プロビジョニング Saga（Keycloak realm 作成 -> DB スキーマ作成 -> 初期設定投入 -> アクティベーション）を実行する
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.tenant.events.v1` にテナントイベント（作成・停止・削除・更新）を配信する

#### プロビジョニング Saga

テナント作成時に以下の 4 ステップを Saga パターンで実行する。各ステップには補償トランザクションが定義されている。

1. Keycloak realm 作成（補償: realm 削除）
2. PostgreSQL スキーマ作成（補償: スキーマ削除）
3. 初期設定投入（補償: 設定削除）
4. テナントステータスを `active` に遷移（補償: `deleted` に遷移）

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_TENANT_`
- テナント名重複は `SYS_TENANT_NAME_CONFLICT`（409）を返却する
- 不正なステータス遷移は `SYS_TENANT_INVALID_STATUS`（400）を返却する
- プロビジョニング失敗時は Saga の補償トランザクションでロールバックし、テナントを `deleted` に遷移する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ステータス遷移バリデーション・テナント名重複チェック | mockall によるリポジトリモック |
| ユースケーステスト | テナント CRUD・メンバー管理 | `usecase_test.rs` でモックリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| Saga テスト | プロビジョニング成功・失敗時のロールバック | モック Keycloak / Saga クライアントで補償トランザクションを検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・テナント状態遷移設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
