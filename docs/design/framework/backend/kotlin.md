# Backend Framework（Kotlin）

k1s0 Backend Framework（Kotlin）は、Ktor ベースのマイクロサービス開発のための共通 Kotlin パッケージ群を提供します。Rust 版・Go 版・C# 版・Python 版と同等の機能を Kotlin で実装しています。

## パッケージ一覧

```
framework/backend/kotlin/
├── build.gradle.kts               # ルートビルド定義
├── settings.gradle.kts             # マルチプロジェクト設定
├── packages/
│   ├── k1s0-error/                # エラー表現の統一（RFC 7807）
│   ├── k1s0-config/               # YAML 設定読み込み
│   ├── k1s0-validation/           # 入力バリデーション
│   ├── k1s0-observability/        # OpenTelemetry 統合
│   ├── k1s0-grpc-server/          # gRPC サーバー共通基盤（grpc-kotlin）
│   ├── k1s0-grpc-client/          # gRPC クライアント共通
│   ├── k1s0-health/               # ヘルスチェック（liveness/readiness）
│   ├── k1s0-db/                   # DB 接続（Exposed + HikariCP）
│   ├── k1s0-domain-event/         # ドメインイベント・Outbox パターン
│   ├── k1s0-resilience/           # サーキットブレーカー・リトライ・タイムアウト
│   ├── k1s0-rate-limit/            # レート制限（トークンバケット・スライディングウィンドウ）
│   ├── k1s0-cache/                # Redis キャッシュ（Lettuce）
│   ├── k1s0-consensus/            # リーダー選出・分散ロック・Saga オーケストレーション
│   └── k1s0-auth/                 # JWT/OIDC 認証（nimbus-jose-jwt）
└── tests/
```

## Tier 構成

### Tier 1（依存なし）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-error | エラー型の統一、RFC 7807 準拠 | kotlinx-serialization |
| k1s0-config | YAML 設定ファイル読み込み・マージ | kaml (YAML for kotlinx-serialization) |
| k1s0-validation | 入力バリデーション | なし |

### Tier 2（Tier 1 のみに依存可）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-observability | 構造化ログ・分散トレーシング・メトリクス | OpenTelemetry SDK, SLF4J |
| k1s0-grpc-server | gRPC サーバー共通設定・インターセプター | grpc-kotlin, protobuf-kotlin |
| k1s0-grpc-client | gRPC クライアントファクトリ・インターセプター | grpc-kotlin |
| k1s0-health | Ktor ヘルスチェックエンドポイント | Ktor Server |
| k1s0-db | DB 接続プール・トランザクション管理 | Exposed, HikariCP, PostgreSQL JDBC |
| k1s0-domain-event | ドメインイベント発行・購読・Outbox | kotlinx-coroutines |
| k1s0-resilience | サーキットブレーカー・リトライ・タイムアウト | resilience4j |
| k1s0-rate-limit | レート制限。トークンバケット・スライディングウィンドウ | kotlinx-coroutines |
| k1s0-cache | Redis キャッシュ操作・キャッシュパターン | Lettuce |
| k1s0-consensus | リーダー選出、分散ロック、Saga オーケストレーション | Exposed, Lettuce, k1s0-db, k1s0-domain-event, k1s0-observability |

### Tier 3（Tier 1, 2 に依存可）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| k1s0-auth | JWT/OIDC 認証・ポリシーベース認可 | nimbus-jose-jwt |

## 技術スタック

| 項目 | 技術 |
|------|------|
| 言語 | Kotlin 2.x |
| Web フレームワーク | Ktor 3.x |
| DI | Koin |
| ORM | Exposed |
| シリアライゼーション | kotlinx-serialization |
| 非同期処理 | kotlinx-coroutines |
| ビルドツール | Gradle (Kotlin DSL) |
| Lint | ktlint |
| 静的解析 | detekt |

## 関連ドキュメント

- [Framework 設計（トップ）](../README.md)
- [サービス構成規約](../../../conventions/service-structure.md)
- [設定・シークレット規約](../../../conventions/config-and-secrets.md)
