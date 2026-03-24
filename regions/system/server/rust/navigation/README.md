# navigation

ナビゲーション・メニュー構造を管理するサーバー。ロールベースのメニュー生成、パンくずリスト、画面遷移権限チェックを提供する。

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
cargo build -p k1s0-navigation-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-navigation-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/navigation/`](../../../../../docs/servers/system/navigation/)
- API 定義: [`api/proto/k1s0/system/navigation/`](../../../../../api/proto/k1s0/system/navigation/)
