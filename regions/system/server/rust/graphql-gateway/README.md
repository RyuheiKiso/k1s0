# graphql-gateway

GraphQL APIゲートウェイサーバー。async-graphqlによるスキーマ定義、DataLoaderによるN+1解消、WebSocketサブスクリプション、gRPCバックエンドへのフェデレーションを担当する。

## 技術スタック

- **言語**: Rust
- **フレームワーク**: Axum（HTTP）/ Tonic（gRPC）
- **設計**: クリーンアーキテクチャ + DDD

## ディレクトリ構造

```
src/
├── domain/        # ドメインモデル・ビジネスロジック
├── usecase/       # ユースケース層
├── adapter/       # 外部アダプター（HTTP/gRPC/DB）
└── infrastructure/ # インフラ設定
```

## ローカル起動

```bash
# 依存サービスを起動
just local-up-dev

# サービス単体をビルド（ワークスペースから）
cargo build -p k1s0-graphql-gateway-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-graphql-gateway-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/graphql-gateway/`](../../../../../docs/servers/system/graphql-gateway/)
