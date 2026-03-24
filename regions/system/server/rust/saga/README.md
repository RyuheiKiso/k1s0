# saga

分散トランザクションをSagaパターンで管理するサーバー。コレオグラフィ/オーケストレーション両方式のSaga実行、補償トランザクション、状態永続化を担当する。

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
cargo build -p k1s0-saga-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-saga-server

# 統合テスト（PostgreSQL, Kafkaが必要）
cargo test -p k1s0-saga-server --features integration-tests

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/saga/`](../../../../../docs/servers/system/saga/)
- API 定義: [`api/proto/k1s0/system/saga/`](../../../../../api/proto/k1s0/system/saga/)
