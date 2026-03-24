# session

セッション管理サーバー。セッションの作成・検証・失効、マルチデバイス対応、セッションストアへのRedis連携を担当する。

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
cargo build -p k1s0-session-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-session-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/session/`](../../../../../docs/servers/system/session/)
- API 定義: [`api/proto/k1s0/system/session/`](../../../../../api/proto/k1s0/system/session/)
