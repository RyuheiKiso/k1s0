# event-store

イベントソーシングのためのイベント永続化サーバー。イベントの書き込み・読み取り・ストリーミング配信、スナップショット管理を担当する。

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
cargo build -p k1s0-event-store-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-event-store-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/event-store/`](../../../../../docs/servers/system/event-store/)
- API 定義: [`api/proto/k1s0/system/eventstore/`](../../../../../api/proto/k1s0/system/eventstore/)
