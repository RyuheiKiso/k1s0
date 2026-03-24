# rule-engine

ビジネスルールの定義・評価サーバー。ルールDSLの解析、条件評価エンジン、ルールバージョン管理、実行結果の監査ログを担当する。

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
cargo build -p k1s0-rule-engine-server
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
cargo test -p k1s0-rule-engine-server

# ワークスペース全体
cargo test --workspace
```

## 設計書

- 設計書: [`docs/servers/system/rule-engine/`](../../../../../docs/servers/system/rule-engine/)
- API 定義: [`api/proto/k1s0/system/ruleengine/`](../../../../../api/proto/k1s0/system/ruleengine/)
