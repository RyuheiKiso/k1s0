# system-ratelimit-server 実装設計

> **注記**: 本ドキュメントは ratelimit-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-ratelimit-server（レートリミットサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（レート制限判定・ルール管理・リセット） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・Redis・キャッシュ・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/ratelimit/)

### ディレクトリ構成

```
regions/system/server/rust/ratelimit/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── rate_limit_rule.rs           # RateLimitRule エンティティ（スコープ・閾値・ウィンドウ）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── rate_limit_repository.rs     # RateLimitRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── rate_limit_domain_service.rs # トークンバケットアルゴリズム制御
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── check_rate_limit.rs              # レート制限チェック
│   │   ├── create_rule.rs                   # ルール作成
│   │   ├── update_rule.rs                   # ルール更新
│   │   ├── delete_rule.rs                   # ルール削除
│   │   ├── get_rule.rs                      # ルール取得
│   │   ├── list_rules.rs                    # ルール一覧
│   │   ├── get_usage.rs                     # 使用量照会
│   │   └── reset_rate_limit.rs              # カウンターリセット
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   └── ratelimit_handler.rs         # axum REST ハンドラー
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── ratelimit_grpc.rs            # gRPC サービス実装（Kong 連携）
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── ratelimit_postgres.rs        # RateLimitRepository PostgreSQL 実装
│   │       └── cached_ratelimit_repository.rs # キャッシュ付きリポジトリ
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ（ルール定義）
│   │   ├── redis_store.rs                   # Redis Lua スクリプトによるトークンバケット実装
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

- **RateLimitDomainService**: トークンバケットアルゴリズムによるレート制限判定ロジック。Redis Lua スクリプトでアトミックに実装する

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CheckRateLimitUseCase` | トークンバケットからトークン消費し、許可/拒否を返す |
| `CreateRuleUseCase` 等 | リミットルールの CRUD |
| `GetUsageUseCase` | 現在の使用量・残余リクエスト数・リセット時刻の取得 |
| `ResetRateLimitUseCase` | 緊急時のカウンターリセット |

#### Redis 連携

- **Redis Store** (`infrastructure/redis_store.rs`): Lua スクリプトによるアトミックなトークンバケット実装。キー形式: `ratelimit:{scope}:{identifier}:{window}`
- Redis 障害時はフェイルオープン（リミットを通過させる）。設定で変更可能

#### Kong 連携

- Kong プラグインから gRPC で `CheckRateLimit` RPC を呼び出すことで、API ゲートウェイレベルのレート制限を実現する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_RATELIMIT_`
- レート制限超過時は 429 Too Many Requests を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | トークンバケットロジック | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| Redis テスト | Lua スクリプト | テスト用 Redis インスタンス |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・Redis Lua スクリプト設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
