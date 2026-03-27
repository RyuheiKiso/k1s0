# system-api-registry-server 実装設計

> **注記**: 本ドキュメントは api-registry-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-api-registry-server（API レジストリサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（スキーマ登録・互換性チェック・差分表示） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・キャッシュ・バリデータ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/api-registry/)

### ディレクトリ構成

```
regions/system/server/rust/api-registry/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── api_registration.rs          # ApiRegistration エンティティ（スキーマ・バージョン）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── api_repository.rs            # ApiRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── api_registry_service.rs      # 破壊的変更検出・差分計算ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── register_schema.rs               # スキーマ新規登録
│   │   ├── register_version.rs              # 新バージョン登録
│   │   ├── get_schema.rs                    # スキーマ取得（最新バージョン）
│   │   ├── get_schema_version.rs            # 特定バージョン取得
│   │   ├── list_schemas.rs                  # スキーマ一覧取得
│   │   ├── list_versions.rs                 # バージョン一覧取得
│   │   ├── delete_version.rs                # バージョン削除
│   │   ├── check_compatibility.rs           # 互換性チェック（破壊的変更検出）
│   │   └── get_diff.rs                      # バージョン間差分取得
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── schema_handler.rs            # axum REST ハンドラー
│   │   │   ├── error.rs                     # エラーレスポンス定義
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── apiregistry_grpc.rs          # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── schema_postgres.rs           # SchemaRepository PostgreSQL 実装
│   │       ├── version_postgres.rs          # VersionRepository PostgreSQL 実装
│   │       └── cached_schema_repository.rs  # キャッシュ付きリポジトリ
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ
│   │   ├── kafka.rs                         # Kafka プロデューサー（スキーマ更新通知）
│   │   ├── startup.rs                       # 起動シーケンス・DI
│   │   └── validator/
│   │       ├── mod.rs
│   │       ├── openapi.rs                   # OpenAPI バリデータ（subprocess）
│   │       └── protobuf.rs                  # Protobuf バリデータ（buf lint subprocess）
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **ApiRegistryService**: スキーマのバージョン間差分計算と破壊的変更検出を行う。フィールド削除・型変更・必須化等の後方互換性破壊を自動検出する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `RegisterSchemaUseCase` | スキーマ新規登録（初回バージョン作成） |
| `RegisterVersionUseCase` | 既存スキーマへの新バージョン登録 |
| `CheckCompatibilityUseCase` | 破壊的変更検出（フィールド削除・型変更等） |
| `GetDiffUseCase` | バージョン間の構造化差分取得 |
| `ListSchemasUseCase` / `ListVersionsUseCase` | 一覧取得 |

#### バリデータ

- **OpenAPI Validator** (`infrastructure/validator/openapi.rs`): openapi-spec-validator を subprocess で呼び出し、OpenAPI 3.x スキーマを検証する
- **Protobuf Validator** (`infrastructure/validator/protobuf.rs`): buf lint を subprocess で呼び出し、Protobuf スキーマを検証する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_APIREG_`
- バリデーション失敗時は 422 Unprocessable Entity を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | 差分計算・互換性チェック | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| バリデータテスト | OpenAPI/Protobuf 検証 | テストスキーマファイルによる検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
