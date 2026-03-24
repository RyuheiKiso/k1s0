# api-registry

APIエンドポイントのメタ情報を一元管理するレジストリサーバー。APIのバージョン管理、スキーマ登録、ディスカバリー機能を提供する。

## 技術スタック

- **言語**: Rust
- **フレームワーク**: Axum（HTTP）
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
cargo build -p k1s0-api-registry-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-api-registry-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/api-registry/`](../../../../../docs/servers/system/api-registry/)
- API 定義: [`api/proto/k1s0/system/apiregistry/`](../../../../../api/proto/k1s0/system/apiregistry/)
