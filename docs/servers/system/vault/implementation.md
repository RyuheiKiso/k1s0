# system-vault-server 実装設計

> **注記**: 本ドキュメントは vault-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-vault-server（シークレット管理サーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト | なし（最内層） |
| usecase | ビジネスロジック（シークレット CRUD・ローテーション・監査ログ） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装・Vault クライアント | usecase, domain |
| infrastructure | 設定・DB接続・キャッシュ・暗号化・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/vault/)

### ディレクトリ構成

```
regions/system/server/rust/vault/
├── src/
│   ├── main.rs                                          # エントリポイント
│   ├── lib.rs                                           # ライブラリルート
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── secret.rs                                # Secret・SecretVersion・SecretValue エンティティ
│   │   │   ├── access_log.rs                            # SecretAccessLog エンティティ
│   │   │   └── access_policy.rs                         # SpiffeAccessPolicy エンティティ
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── secret_store.rs                          # SecretStore トレイト
│   │       └── access_log_repo.rs                       # AccessLogRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── get_secret.rs                                # シークレット取得（バージョン指定可能）
│   │   ├── set_secret.rs                                # シークレット作成・更新（バージョン自動インクリメント）
│   │   ├── delete_secret.rs                             # シークレット削除
│   │   ├── list_secrets.rs                              # シークレットパス一覧
│   │   ├── rotate_secret.rs                             # シークレットローテーション
│   │   └── list_audit_logs.rs                           # 監査ログ一覧
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── vault_handler.rs                         # axum REST ハンドラー
│   │   │   └── health.rs                                # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── vault_grpc.rs                            # gRPC サービス実装
│   │   │   └── tonic_service.rs                         # tonic サービスラッパー
│   │   ├── gateway/
│   │   │   ├── mod.rs
│   │   │   └── vault_client.rs                          # HashiCorp Vault クライアント（vaultrs 経由）
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                                  # JWT 認証ミドルウェア
│   │   │   ├── rbac.rs                                  # RBAC ミドルウェア
│   │   │   └── spiffe.rs                                # SPIFFE ID ベース認可ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── vault_secret_store.rs                    # SecretStore HashiCorp Vault 実装
│   │       ├── secret_store_postgres.rs                 # SecretStore PostgreSQL フォールバック実装
│   │       ├── cached_secret_store.rs                   # キャッシュ付き SecretStore ラッパー
│   │       └── access_log_postgres.rs                   # AccessLogRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                                    # 設定構造体・読み込み
│   │   ├── database.rs                                  # PostgreSQL 接続プール
│   │   ├── cache.rs                                     # moka キャッシュ（リース期限 80% TTL）
│   │   ├── encryption.rs                                # シークレットデータ暗号化
│   │   ├── kafka_producer.rs                            # Kafka プロデューサー（アクセス監査・ローテーション通知）
│   │   └── startup.rs                                   # 起動シーケンス・DI
│   └── proto/                                           # tonic-build 生成コード
│       ├── mod.rs
│       ├── k1s0.system.vault.v1.rs
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

#### ドメインモデル

- **Secret**: バージョン管理付きシークレット。`new(path, data)` で初期バージョン作成、`update(data)` で新バージョン追加、`get_version(version)` でバージョン指定取得（destroyed 済みは None）
- **SecretAccessLog**: シークレットアクセスの監査ログ。SPIFFE ID・アクション種別・成功/失敗を記録する
- **SpiffeAccessPolicy**: SPIFFE ID ベースのアクセスポリシー。glob パターンでシークレットパスを照合する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `GetSecretUseCase` | シークレット取得（バージョン指定可能）・キャッシュ利用・監査ログ記録 |
| `SetSecretUseCase` | シークレット作成・更新（バージョン自動インクリメント）・監査ログ記録 |
| `DeleteSecretUseCase` | シークレット削除・キャッシュ無効化・監査ログ記録 |
| `ListSecretsUseCase` | パスプレフィックスによるシークレット一覧取得 |
| `RotateSecretUseCase` | シークレットローテーション・Kafka 通知発行 |
| `ListAuditLogsUseCase` | 監査ログ一覧取得（オフセット/リミット） |

#### 外部連携

- **HashiCorp Vault** (`adapter/gateway/vault_client.rs`): vaultrs クレートで HashiCorp Vault KV v2 に接続する
- **SPIFFE 認可** (`adapter/middleware/spiffe.rs`): SPIFFE ID ベースのアクセス制御。`SpiffeAccessPolicy` のパターンマッチングで許可/拒否を判定する
- **Kafka Producer** (`infrastructure/kafka_producer.rs`): `k1s0.system.vault.access.v1`（アクセス監査）および `k1s0.system.vault.secret_rotated.v1`（ローテーション通知）にイベントを配信する
- **暗号化** (`infrastructure/encryption.rs`): PostgreSQL フォールバック時のシークレットデータ暗号化を担当する

#### キャッシュ戦略

- moka で TTL ベースのインメモリキャッシュを実装する
- TTL はリース期限の 80%（例: リース 1 時間 -> TTL 48 分）
- 最大エントリ数: 10,000
- キャッシュキー: `{path}:{version}`
- エビクションポリシー: TTL 期限切れ + LRU

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_VAULT_`
- SPIFFE ID によるアクセス拒否は `SYS_VAULT_ACCESS_DENIED`（403）を返却する
- HashiCorp Vault 接続障害時は `SYS_VAULT_UPSTREAM_ERROR`（502）を返却する
- すべてのシークレットアクセス（成功・失敗問わず）を監査ログに記録する

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | Secret バージョン管理・SPIFFE パターンマッチング | ドメインモデルの直接テスト |
| ユースケーステスト | シークレット CRUD・ローテーション | `usecase_test.rs` でモックリポジトリを使用 |
| 統合テスト | REST/gRPC ハンドラー | `integration_test.rs` で axum-test / tonic テストクライアント |
| キャッシュテスト | TTL 動作・エビクション | moka キャッシュの TTL 期限切れ・LRU エビクションを検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・キャッシュ設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
