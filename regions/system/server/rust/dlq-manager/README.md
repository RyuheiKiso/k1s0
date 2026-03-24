# dlq-manager

デッドレターキュー（DLQ）を管理するサーバー。処理失敗メッセージの蓄積、再試行スケジューリング、エラー分析・可視化を担当する。

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
cargo build -p k1s0-dlq-manager
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-dlq-manager

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/dlq-manager/`](../../../../../docs/servers/system/dlq-manager/)
- API 定義: [`api/proto/k1s0/system/dlq/`](../../../../../api/proto/k1s0/system/dlq/)
