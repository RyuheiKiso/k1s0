# Justfile — 統一ビルドオーケストレーション

## 概要

CI (`ci.yaml`) と同一のモジュール探索パターン・ビルド手順をローカルで実行するための just レシピ集。
`rg` (ripgrep) でモノリポ内の `regions/` と `CLI/` からモジュールを自動探索し、各言語のツールチェーンで lint / test / build / format を実行する。

## 前提条件

| ツール | バージョン | 用途 |
|--------|-----------|------|
| [just](https://github.com/casey/just) | >= 1.0 | コマンドランナー |
| [rg](https://github.com/BurntSushi/ripgrep) (ripgrep) | >= 14.0 | モジュール探索 |
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

justfile のモジュール探索パターンは CI (`ci.yaml`) と完全に一致している。

- **Go**: `rg --files -g 'go.mod' regions CLI | sort` で `go.mod` を持つ全ディレクトリを探索
- **Rust**: `rg --files -g 'Cargo.toml' regions CLI | sort` で探索し、ワークスペースルート (`CLI/Cargo.toml`, `regions/system/Cargo.toml`, `CLI/crates/k1s0-gui/Cargo.toml`) および実験系クレート (`regions/system/server/rust/ai-agent/Cargo.toml`, `regions/system/server/rust/ai-gateway/Cargo.toml`) をスキップ
- **TypeScript**: `rg --files -g 'package.json' regions CLI | sort` で `package.json` を持つディレクトリを探索
- **Dart**: `rg --files -g 'pubspec.yaml' regions CLI | sort` で `pubspec.yaml` を持つディレクトリを探索し、Flutter かどうかを `sdk: flutter` の有無で判定

CI の探索ロジックを変更した場合は、justfile も同時に更新すること。
