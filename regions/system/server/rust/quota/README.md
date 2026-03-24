# quota

リソース使用量の制限・管理サーバー。テナントごとのクォータ設定、使用量カウント・追跡、超過時のエラーレスポンスを担当する。

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
cargo build -p k1s0-quota-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-quota-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/quota/`](../../../../../docs/servers/system/quota/)
- API 定義: [`api/proto/k1s0/system/quota/`](../../../../../api/proto/k1s0/system/quota/)
