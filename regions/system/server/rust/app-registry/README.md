# app-registry

アプリケーション・マイクロサービスのメタ情報を管理するレジストリサーバー。サービスの登録、設定管理、依存関係の追跡を担当する。

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
cargo build -p k1s0-app-registry
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-app-registry

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/app-registry/`](../../../../../docs/servers/system/app-registry/)
