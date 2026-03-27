# system-policy-server 実装設計

> **注記**: 本ドキュメントは policy-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-policy-server（ポリシーサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（ポリシー管理・評価・バンドル管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・OPAクライアント・キャッシュ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/policy/)

### ディレクトリ構成

```
regions/system/server/rust/policy/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── policy.rs                    # Policy エンティティ（Rego ポリシー本文）
│   │   │   ├── policy_bundle.rs             # PolicyBundle エンティティ
│   │   │   └── policy_evaluation.rs         # PolicyEvaluation エンティティ（評価結果）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── policy_repository.rs         # PolicyRepository トレイト
│   │   │   └── bundle_repository.rs         # BundleRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── policy_domain_service.rs     # ポリシー評価フロー制御
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_policy.rs                 # ポリシー作成
│   │   ├── update_policy.rs                 # ポリシー更新
│   │   ├── delete_policy.rs                 # ポリシー削除
│   │   ├── get_policy.rs                    # ポリシー取得
│   │   ├── list_policies.rs                 # ポリシー一覧
│   │   ├── evaluate_policy.rs               # ポリシー評価（OPA 経由）
│   │   ├── create_bundle.rs                 # バンドル作成
│   │   ├── get_bundle.rs                    # バンドル取得
│   │   └── list_bundles.rs                  # バンドル一覧
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── policy_handler.rs            # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── policy_grpc.rs               # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── policy_postgres.rs           # PolicyRepository PostgreSQL 実装
│   │       ├── bundle_postgres.rs           # BundleRepository PostgreSQL 実装
│   │       └── cached_policy_repository.rs  # キャッシュ付き PolicyRepository
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ（評価結果 TTL 30秒）
│   │   ├── opa_client.rs                    # OPA HTTP API クライアント
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（ポリシー変更通知）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **PolicyDomainService**: ポリシー評価のフロー制御。OPA HTTP API（`/v1/data/{package_path}`）を呼び出し、allow/deny 判定を返却する。OPA 未設定時は `policy.enabled` フラグでフォールバック評価を行う

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreatePolicyUseCase` / `UpdatePolicyUseCase` / `DeletePolicyUseCase` | Rego ポリシーの CRUD |
| `EvaluatePolicyUseCase` | 入力データに対するポリシー評価（OPA HTTP API 経由） |
| `CreateBundleUseCase` / `GetBundleUseCase` / `ListBundlesUseCase` | ポリシーバンドル管理 |

#### 外部連携

- **OPA Client** (`infrastructure/opa_client.rs`): OPA HTTP API を呼び出してポリシー評価を行う
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): ポリシー変更時に `k1s0.system.policy.updated.v1` トピックに通知する

#### キャッシュ戦略

- moka で評価結果を TTL 30 秒キャッシュする
- Kafka 通知受信時にキャッシュを即座に無効化する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_POLICY_`
- OPA 接続障害時はフォールバック評価を行う

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ポリシー評価フロー | mockall によるリポジトリ・OPA クライアントモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| OPA テスト | ポリシー評価 | テスト用 OPA サーバーまたはモック |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・OPA 連携設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
