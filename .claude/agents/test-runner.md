# テスト実行エージェント

各コンポーネントのテスト実行を支援するエージェント。

## Rust テスト

### CLI

```bash
cd CLI

# 全テスト実行
cargo test --all

# 特定クレートのテスト
cargo test -p k1s0-cli
cargo test -p k1s0-generator
cargo test -p k1s0-lsp

# 特定テストのみ
cargo test <test_name>

# 出力を表示
cargo test -- --nocapture

# ドキュメントテスト
cargo test --doc
```

### Framework (Rust)

```bash
cd framework/backend/rust

# ワークスペース全体
cargo test --workspace

# 特定クレート
cargo test -p k1s0-error
cargo test -p k1s0-config
cargo test -p k1s0-validation
cargo test -p k1s0-auth
# ... 他の crate

# 特定サービス
cargo test -p auth-service
cargo test -p config-service
cargo test -p endpoint-service
```

### テストオプション

```bash
# 並列実行を制限（競合防止）
cargo test -- --test-threads=1

# 失敗したテストのみ再実行
cargo test -- --failed

# 特定のテストをフィルタ
cargo test <pattern>
```

## React テスト

```bash
cd framework/frontend/react

# 全テスト実行
pnpm test

# Watch モード
pnpm test --watch

# カバレッジ付き
pnpm test --coverage

# 特定パッケージ
pnpm --filter @k1s0/navigation test
pnpm --filter @k1s0/config test
```

## CI テスト

GitHub Actions で実行されるテスト:

### rust.yml

```yaml
- cargo build --all
- cargo test --all
- cargo clippy --all-targets -- -D warnings
```

### generation.yml

```yaml
- k1s0 lint
- テンプレート生成の検証
- manifest.json バリデーション
```

## テストカバレッジ

### Rust (cargo-llvm-cov)

```bash
# llvm-cov インストール
cargo install cargo-llvm-cov

# カバレッジ測定
cargo llvm-cov --workspace

# HTML レポート生成
cargo llvm-cov --workspace --html
```

### React (Jest)

```bash
cd framework/frontend/react
pnpm test --coverage
# レポート: coverage/lcov-report/index.html
```

## テスト規約

1. **単体テスト**: 各モジュールの `#[cfg(test)]` ブロック内
2. **統合テスト**: `tests/` ディレクトリ
3. **E2E テスト**: 別途定義（将来対応）

### テスト命名

```rust
#[test]
fn test_<what>_<scenario>_<expected>() {
    // Given-When-Then パターン
}
```

例: `test_validate_config_empty_returns_error`
