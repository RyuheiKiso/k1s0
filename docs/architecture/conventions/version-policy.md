# バージョン管理ポリシー

## 目的

各言語・フレームワークのバージョンを一元管理し、README.md と実態の乖離を防止する。

## Source of Truth（信頼すべき情報源）

| 言語/ツール | Source of Truth | ファイルパス例 |
|------------|----------------|---------------|
| Go | `go.mod` の `go` ディレクティブ | `regions/system/server/go/bff-proxy/go.mod` |
| Rust | `Cargo.toml` の `rust-version` / `rustfmt.toml` | `CLI/crates/k1s0-cli/Cargo.toml` |
| Dart | `pubspec.yaml` の `environment.sdk` | `regions/system/client/flutter/system_client/pubspec.yaml` |
| Flutter | `pubspec.yaml` の `environment.flutter` | 同上 |
| Node.js | `package.json` の `engines.node` | `regions/system/client/react/system-client/package.json` |
| TypeScript | `package.json` の `devDependencies.typescript` | 同上 |

## バージョン更新ルール

### 1. Source of Truth を先に更新する

バージョンを上げる場合は、必ず上記の Source of Truth ファイルを最初に更新する。
CI/CD やビルドはこれらのファイルを参照するため、ここが正となる。

### 2. README.md を同期する

Source of Truth を更新したら、`README.md` の「技術スタック」テーブルおよびバッジも合わせて更新する。

**更新対象箇所**:
- バッジ画像 URL 内のバージョン番号（例: `Go-1.24-00ADD8`）
- 「技術スタック > 言語・フレームワーク」テーブルのバージョン列
- 「クイックスタート > 前提条件」に記載されたバージョン

### 3. 1 コミットで同時更新する

Source of Truth と README.md は同一のコミット（またはプルリクエスト）で更新する。
別々にすると乖離期間が発生するため、必ず同時に変更する。

## メジャーバージョン更新時の追加チェック

- [ ] CI/CD ワークフロー（`.github/workflows/`）のバージョン指定を更新
- [ ] `docker-compose.yaml` のビルド引数を更新（該当する場合）
- [ ] `.devcontainer/` の設定を更新（該当する場合）
- [ ] 破壊的変更の有無を確認し、影響範囲をドキュメント化

## バージョン確認コマンド

```bash
# Go バージョン確認
grep -r "^go " regions/system/server/go/*/go.mod

# Dart SDK バージョン確認
grep -A1 "environment:" regions/system/client/flutter/*/pubspec.yaml

# Rust バージョン確認
grep "rust-version" CLI/crates/*/Cargo.toml

# Node.js バージョン確認
grep -A1 '"engines"' regions/system/client/react/*/package.json
```
