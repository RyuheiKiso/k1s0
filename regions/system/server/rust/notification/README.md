# notification

通知配信を担当するサーバー。メール・プッシュ・SMS・Webhookの多チャンネル配信、テンプレート管理、配信履歴追跡を提供する。

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
cargo build -p k1s0-notification-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-notification-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/notification/`](../../../../../docs/servers/system/notification/)
- API 定義: [`api/proto/k1s0/system/notification/`](../../../../../api/proto/k1s0/system/notification/)
