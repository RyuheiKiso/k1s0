# service-catalog

サービスカタログを管理するサーバー。利用可能なサービス・機能の一覧管理、サービス検索、プランごとの機能可用性管理を担当する。

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
cargo build -p k1s0-service-catalog
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-service-catalog

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/service-catalog/`](../../../../../docs/servers/system/service-catalog/)
- API 定義: [`api/proto/k1s0/system/servicecatalog/`](../../../../../api/proto/k1s0/system/servicecatalog/)
