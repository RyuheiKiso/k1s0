# policy

アクセスポリシーの定義・評価サーバー。ABAC/RBACポリシーの管理、認可判定エンジン、ポリシーバージョン管理を担当する。

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
cargo build -p k1s0-policy-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-policy-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/policy/`](../../../../../docs/servers/system/policy/)
- API 定義: [`api/proto/k1s0/system/policy/`](../../../../../api/proto/k1s0/system/policy/)
