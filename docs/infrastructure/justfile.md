# Justfile — 統一ビルドオーケストレーション

## 概要

CI (`ci.yaml`) と同一のビルド手順をローカルで実行するための just レシピ集。
`modules.yaml`（モジュールレジストリ）を `scripts/list-modules.sh` で読み取り、各言語のツールチェーンで lint / test / build / format を実行する。
Rust / Go はワークスペース一括操作を採用し、O(1) の起動回数でビルドを完了する。

## 前提条件

| ツール | バージョン | 用途 |
|--------|-----------|------|
| [just](https://github.com/casey/just) | >= 1.0 | コマンドランナー |
| Go | 1.24 | Go ビルド・テスト |
| golangci-lint | latest | Go リント |
| Rust (cargo) | 1.93 | Rust ビルド・テスト |
| Node.js (npm) | 22 | TypeScript ビルド・テスト |
| Flutter | 3.24 | Dart/Flutter ビルド・テスト |

## 使い方

```bash
# レシピ一覧を表示
just

# 全言語の lint を実行
just lint

# 特定言語のみ実行
just lint-go
just test-rust
just fmt-ts

# CI と同等の全チェック (lint + test + build)
just ci

# フォーマット修正
just fmt

# Proto コード生成
just proto
```

## レシピ一覧

| レシピ | 説明 |
|--------|------|
| `lint` | 全言語リント (`lint-go` + `lint-rust` + `lint-ts` + `lint-dart`) |
| `lint-go` | Go: `golangci-lint run ./...` + `go vet ./...` |
| `lint-rust` | Rust: `cargo fmt --check` + `cargo clippy -D warnings` |
| `lint-ts` | TypeScript: `npm run lint` + `npm run typecheck` |
| `lint-dart` | Dart/Flutter: `dart analyze` / `flutter analyze` |
| `test` | 全言語テスト (`test-go` + `test-rust` + `test-ts` + `test-dart`) |
| `test-go` | Go: `go test ./... -race -count=1` |
| `test-rust` | Rust: `cargo test --all` |
| `test-ts` | TypeScript: `npm test` |
| `test-dart` | Dart/Flutter: `dart test` / `flutter test` |
| `fmt` | 全言語フォーマット (`fmt-go` + `fmt-rust` + `fmt-ts` + `fmt-dart`) |
| `fmt-go` | Go: `gofmt -w .` |
| `fmt-rust` | Rust: `cargo fmt --all` |
| `fmt-ts` | TypeScript: `npm run format` |
| `fmt-dart` | Dart: `dart format lib/ test/` |
| `build` | 全言語ビルド (`build-go` + `build-rust` + `build-ts`) |
| `build-go` | Go: `go build ./...` |
| `build-rust` | Rust: `cargo build --all-targets` |
| `build-ts` | TypeScript: `npm run build` |
| `proto` | Proto コード生成 (`scripts/generate-proto.sh`) |
| `ci` | CI 全実行 (`lint` + `test` + `build`) |
| `security` | 全言語セキュリティスキャン (`security-go` + `security-rust` + `security-ts` + `security-dart`) |
| `security-go` | Go: `govulncheck ./...`（全モジュール自動探索） |
| `security-rust` | Rust: `cargo audit`（CLI/ + regions/system/ ワークスペース） |
| `security-ts` | TypeScript: `npm audit --audit-level=high`（全 package-lock.json 自動探索） |
| `security-dart` | Dart/Flutter: `pub outdated`（advisory、全 pubspec.yaml 自動探索） |

## CI との整合性

justfile と CI (`ci.yaml`) はともに `modules.yaml`（モジュールレジストリ）を唯一の情報源として使用する。

### モジュール探索

- **全言語共通**: `scripts/list-modules.sh --lang <LANG> --no-skip-ci` で `modules.yaml` から CI 対象モジュールを取得
- **スキップ対象**: `modules.yaml` の `skip-ci: true` フラグで管理（ワークスペースルート、proto ディレクトリ等）
- **experimental クレート**: `modules.yaml` の `status: experimental` で管理（CI では各クレートの `Cargo.toml` から実際の package name を取得し `--exclude` に変換。除外対象が workspace に存在しない場合はエラー）

### ワークスペース一括操作

- **Rust**: `cargo fmt/clippy/test/build --manifest-path regions/system/Cargo.toml --workspace` でワークスペース一括実行。experimental クレートは `Cargo.toml` の package name ベースで `--exclude` 除外
- **Go**: `go build ./...` で `go.work` 経由の一括ビルド
- **TypeScript / Dart**: 個別モジュール反復（ワークスペース機構がないため）

### 新サービス追加時

1. `modules.yaml` にエントリを追加する
2. justfile・CI の両方に自動的に反映される（コード変更不要）
