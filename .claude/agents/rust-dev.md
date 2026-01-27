# Rust 開発エージェント

Rust コードの開発、ビルド、テスト、品質チェックを支援するエージェント。

## 対象領域

- `CLI/` - k1s0 CLI (k1s0-cli, k1s0-generator, k1s0-lsp)
- `framework/backend/rust/` - 共通 Rust crates とサービス

## 主な操作

### ビルド・テスト

```bash
# CLI のビルド
cd CLI && cargo build

# CLI のテスト
cd CLI && cargo test --all

# Framework のビルド
cd framework/backend/rust && cargo build --workspace

# Framework のテスト
cd framework/backend/rust && cargo test --workspace
```

### コード品質

```bash
# Clippy（lint）
cargo clippy --all-targets --all-features -- -D warnings

# フォーマット確認
cargo fmt --check

# フォーマット適用
cargo fmt
```

### 依存関係

```bash
# 依存関係の確認
cargo tree

# 未使用依存の検出
cargo +nightly udeps
```

## Crate 依存階層

### CLI ワークスペース (CLI/)

- `k1s0-cli`: 実行 CLI（clap ベース）
- `k1s0-generator`: テンプレート展開・差分適用・Lint エンジン
- `k1s0-lsp`: Language Server Protocol 実装

### Framework ワークスペース (framework/backend/rust/)

**Tier 1（コア基盤）:**
- `k1s0-error`: エラー表現の統一
- `k1s0-config`: 設定読み込み
- `k1s0-validation`: 入力バリデーション
- `k1s0-observability`: ログ/トレース/メトリクス

**Tier 2（通信基盤）:**
- `k1s0-grpc-server`: gRPC サーバ基盤
- `k1s0-grpc-client`: gRPC クライアント基盤
- `k1s0-resilience`: Timeout/Bulkhead/CircuitBreaker

**Tier 3（ビジネスロジック支援）:**
- `k1s0-auth`: JWT/OIDC 検証、ポリシー評価
- `k1s0-db`: DB 接続プール、トランザクション
- `k1s0-cache`: Redis クライアント
- `k1s0-health`: Kubernetes プローブ対応

**共通サービス:**
- `auth-service`: 認証・認可マイクロサービス
- `config-service`: 動的設定マイクロサービス
- `endpoint-service`: エンドポイント管理マイクロサービス

## 規約

- `unsafe_code = "forbid"` - unsafe コードは禁止
- Clippy の `pedantic` レベルを有効化
- Clean Architecture の依存方向を遵守
