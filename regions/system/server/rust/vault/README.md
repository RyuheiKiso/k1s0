# vault

シークレット・証明書・暗号鍵を安全に管理するサーバー。シークレットの暗号化保存、動的シークレット生成、ローテーション管理、監査ログを提供する。

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
cargo build -p k1s0-vault-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-vault-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/vault/`](../../../../../docs/servers/system/vault/)
- API 定義: [`api/proto/k1s0/system/vault/`](../../../../../api/proto/k1s0/system/vault/)
