# Backend Framework

k1s0 Backend Framework は、マイクロサービス開発のための共通ライブラリ群を提供します。各パッケージは独立して使用可能で、Clean Architecture の原則に従って設計されています。

## 言語別パッケージ

- Rust - Rust crate 群（本 README 下部に記載）
- [Go](./go.md) - Go パッケージ群
- [C#](./csharp.md) - NuGet パッケージ群
- [Python](./python.md) - Python パッケージ群
- [Kotlin](./kotlin.md) - Kotlin パッケージ群

---

## Rust Crate 群

## Crate 一覧

```
framework/backend/rust/crates/
├── k1s0-error/         # エラー表現の統一
├── k1s0-config/        # 設定読み込み
├── k1s0-validation/    # 入力バリデーション
├── k1s0-observability/ # ログ/トレース/メトリクス
├── k1s0-grpc-server/   # gRPC サーバ共通基盤
├── k1s0-grpc-client/   # gRPC クライアント共通
├── k1s0-resilience/    # レジリエンスパターン
├── k1s0-rate-limit/    # レート制限（トークンバケット、スライディングウィンドウ）
├── k1s0-health/        # ヘルスチェック
├── k1s0-cache/         # Redis キャッシュ
├── k1s0-db/            # DB 接続・トランザクション
├── k1s0-domain-event/  # ドメインイベント発行・購読・Outbox
├── k1s0-consensus/     # リーダー選出・分散ロック・Saga オーケストレーション
└── k1s0-auth/          # 認証・認可
```

## 各 crate 設計

- [k1s0-error](k1s0-error.md)
- [k1s0-config](k1s0-config.md)
- [k1s0-validation](k1s0-validation.md)
- [k1s0-observability](k1s0-observability.md)
- [k1s0-grpc-server](k1s0-grpc-server.md)
- [k1s0-grpc-client](k1s0-grpc-client.md)
- [k1s0-resilience](k1s0-resilience.md)
- [k1s0-rate-limit](k1s0-rate-limit.md)
- [k1s0-health](k1s0-health.md)
- [k1s0-cache](k1s0-cache.md)
- [k1s0-db](k1s0-db.md)
- [k1s0-domain-event](k1s0-domain-event.md)
- [k1s0-consensus](k1s0-consensus.md)
- [k1s0-auth](k1s0-auth.md)

---

## 依存関係

```
k1s0-error          # 基盤（依存なし）
k1s0-config         # 基盤（依存なし）
k1s0-validation     # 基盤（依存なし）
k1s0-observability  # 基盤（依存なし）
k1s0-resilience     # 基盤（依存なし）

k1s0-rate-limit     # インフラ
  └── k1s0-config (feature="config")

k1s0-grpc-server    # インフラ
  ├── k1s0-error
  └── k1s0-observability

k1s0-grpc-client    # インフラ（依存なし）

k1s0-health         # インフラ（依存なし）

k1s0-cache          # 業務
  └── k1s0-health (feature="health")

k1s0-db             # 業務
  └── sqlx (feature="postgres")

k1s0-domain-event   # 業務
  └── k1s0-db (feature="outbox")

k1s0-consensus      # 業務
  ├── k1s0-db
  ├── k1s0-domain-event
  └── k1s0-observability

k1s0-auth           # 業務
  ├── k1s0-cache (feature="redis-cache")
  ├── k1s0-db (feature="postgres-policy")
  ├── axum, tower (feature="axum-layer")
  └── tonic (feature="tonic-interceptor")
```
