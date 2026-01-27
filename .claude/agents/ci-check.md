# CI/CD チェックエージェント

CI パイプラインのローカル実行とチェックを支援するエージェント。

## GitHub Actions ワークフロー

### 1. rust.yml - Rust CI

**トリガー**: Rust ファイルの変更時

```bash
# ローカルで同等のチェック実行
cd CLI && cargo build && cargo test --all && cargo clippy --all-targets -- -D warnings

cd framework/backend/rust && cargo build --workspace && cargo test --workspace && cargo clippy --all-targets -- -D warnings
```

### 2. buf.yml - Protocol Buffers lint

**トリガー**: .proto ファイルの変更時

```bash
# ローカルで実行
./scripts/buf-check.sh

# または
buf lint
```

### 3. openapi.yml - OpenAPI lint

**トリガー**: OpenAPI 仕様ファイルの変更時

```bash
# ローカルで実行
./scripts/openapi-check.sh

# または
npx @stoplight/spectral-cli lint <file>.yaml
```

### 4. generation.yml - テンプレート生成検証

**トリガー**: テンプレートまたは CLI の変更時

```bash
# ローカルで実行
./scripts/gen-check.sh

# または手動で
cd CLI && cargo build
./target/debug/k1s0 lint
```

## 全 CI チェックのローカル実行

```bash
# Rust チェック
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all

# Protocol Buffers チェック
./scripts/buf-check.sh

# OpenAPI チェック
./scripts/openapi-check.sh

# k1s0 lint
./scripts/gen-check.sh
```

## スクリプト一覧

| スクリプト | 説明 |
|-----------|------|
| `scripts/buf-check.sh` | Protocol Buffers lint 検証 |
| `scripts/gen-check.sh` | テンプレート生成検証 |
| `scripts/openapi-check.sh` | OpenAPI 仕様 lint 検証 |

## トラブルシューティング

### CI で失敗した場合

1. **ローカルで同じチェックを実行**
2. **エラーメッセージを確認**
3. **修正してコミット**

### よくある失敗原因

| 失敗 | 原因 | 対処 |
|------|------|------|
| Clippy warning | コード品質問題 | `cargo clippy --fix` |
| Test failure | テストの失敗 | テストコードを確認 |
| buf lint | proto 定義の問題 | buf の出力を確認 |
| Spectral error | OpenAPI 仕様の問題 | 必須フィールドを追加 |
| k1s0 lint | 規約違反 | `k1s0 lint --fix` |

## 依存ツールのインストール

```bash
# buf (Protocol Buffers)
# https://buf.build/docs/installation

# Spectral (OpenAPI)
npm install -g @stoplight/spectral-cli

# Rust ツール
rustup component add clippy
```
