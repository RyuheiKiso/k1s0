# event-monitor

イベントストリームのリアルタイム監視・アラートを担当するサーバー。異常検知、イベントフィルタリング、通知トリガーを提供する。

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
cargo build -p k1s0-event-monitor-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-event-monitor-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/event-monitor/`](../../../../../docs/servers/system/event-monitor/)
- API 定義: [`api/proto/k1s0/system/eventmonitor/`](../../../../../api/proto/k1s0/system/eventmonitor/)
