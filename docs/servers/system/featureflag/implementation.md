# system-featureflag-server 実装設計

> **注記**: 本ドキュメントは featureflag-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-featureflag-server（フィーチャーフラグサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（フラグ管理・評価・監査ログ） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・キャッシュ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/featureflag/)

### ディレクトリ構成

```
regions/system/server/rust/featureflag/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── feature_flag.rs              # FeatureFlag エンティティ（キー・バリアント・ルール）
│   │   │   ├── evaluation.rs                # Evaluation エンティティ（評価結果）
│   │   │   └── flag_audit_log.rs            # FlagAuditLog エンティティ（変更監査ログ）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── flag_repository.rs           # FlagRepository トレイト
│   │   │   └── flag_audit_log_repository.rs # FlagAuditLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── feature_flag_domain_service.rs # フラグ評価ロジック（バリアント/ルール制御）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_flag.rs                   # フラグ作成
│   │   ├── update_flag.rs                   # フラグ更新
│   │   ├── delete_flag.rs                   # フラグ削除
│   │   ├── get_flag.rs                      # フラグ詳細取得
│   │   ├── list_flags.rs                    # フラグ一覧取得
│   │   ├── evaluate_flag.rs                 # フラグ評価（属性ベース）
│   │   └── watch_feature_flag.rs            # フラグ変更監視（gRPC Stream）
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── flag_handler.rs              # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── featureflag_grpc.rs          # gRPC サービス実装
│   │   │   ├── tonic_service.rs             # tonic サービスラッパー
│   │   │   └── watch_stream.rs              # gRPC Server Stream（フラグ変更通知）
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── featureflag_postgres.rs      # FlagRepository PostgreSQL 実装
│   │       ├── cached_featureflag_repository.rs # キャッシュ付き FlagRepository
│   │       └── flag_audit_log_postgres.rs   # FlagAuditLogRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ（TTL 60秒）
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（変更通知）
│   │   ├── kafka_consumer.rs                # Kafka コンシューマー（キャッシュ無効化）
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

- **FeatureFlagDomainService**: バリアント（重み付き値）とルール（属性マッチング → バリアント選択）によるフラグ評価ロジック。ユーザー・テナント・属性に基づくサーバー側評価を行う

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateFlagUseCase` / `UpdateFlagUseCase` / `DeleteFlagUseCase` | フラグ定義の CRUD |
| `EvaluateFlagUseCase` | ユーザー・テナント・属性に基づくフラグ評価 |
| `WatchFeatureFlagUseCase` | gRPC Server Stream によるフラグ変更のリアルタイム通知 |

#### テナント分離（STATIC-CRITICAL-001）

全データアクセスはテナントスコープで分離される。

| レイヤー | 実装 |
|---------|------|
| DB | 全テーブルに `tenant_id UUID NOT NULL`、全クエリに `WHERE tenant_id = $X` |
| リポジトリトレイト | 全メソッドの第1引数が `tenant_id: Uuid` |
| ユースケース入力 | `CreateFlagInput` / `UpdateFlagInput` / `EvaluateFlagInput` に `tenant_id: Uuid` |
| gRPC adapter | ADR-0028 Phase 1: `x-tenant-id` gRPC メタデータから取得し、未設定時はシステムテナント UUID にフォールバック。`tenant_id_from_metadata()` ヘルパーを使用 |
| REST ハンドラー | `Option<Extension<k1s0_auth::Claims>>` から `tenant_id` を抽出、未認証時はシステムテナント UUID をフォールバックに使用 |
| キャッシュキー | `{tenant_id}:{flag_key}` 形式（テナント間のキャッシュ汚染を防ぐ） |
| Kafka キャッシュ無効化 | イベントペイロードに `tenant_id` フィールドを含め、対象テナントのキャッシュのみ無効化 |

- システムテナント UUID: `00000000-0000-0000-0000-000000000001`
- ADR-0028 Phase 2 完了後: gRPC メタデータの `x-tenant-id` を必須化し、フォールバックを廃止する

#### キャッシュ戦略

- moka で評価結果を TTL 60 秒キャッシュする（キャッシュキー: `{tenant_id}:{flag_key}`）
- Kafka `k1s0.system.featureflag.changed.v1` 通知受信時にキャッシュを即座に無効化する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_FF_`
- フラグ未発見時は 404 を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | フラグ評価・バリアント選択 | mockall によるリポジトリモック |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| キャッシュテスト | moka キャッシュ無効化 | Kafka 通知をシミュレートして検証 |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・バリアント/ルール設計
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
