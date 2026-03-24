# auth

認証・認可を担当するサーバー。JWTトークン発行・検証、OAuth2/OIDCフロー、パーミッション管理を提供する。

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
cargo build -p k1s0-auth-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-auth-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/auth/`](../../../../../docs/servers/system/auth/)
- API 定義: [`api/proto/k1s0/system/auth/`](../../../../../api/proto/k1s0/system/auth/)
