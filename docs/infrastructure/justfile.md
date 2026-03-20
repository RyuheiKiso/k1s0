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

## 技術監査対応の改善事項

### DRY 化: ワークスペース除外ロジックの集約

実験系クレート（`ai-agent`, `ai-gateway` 等）のワークスペース除外ロジックを `scripts/list-experimental-excludes.sh` に集約した。justfile と CI の両方がこのスクリプトを呼び出すことで、除外対象の定義が一箇所に集約され、追加・削除時の変更漏れを防止する。

```bash
# scripts/list-experimental-excludes.sh の出力例:
# --exclude ai-agent --exclude ai-gateway
EXCLUDES=$(bash scripts/list-experimental-excludes.sh)
cargo clippy --manifest-path regions/system/Cargo.toml --workspace $EXCLUDES -- -D warnings
```

### standalone サーバー変数化

justfile 内でハードコードされていた standalone サーバーのパスを `_standalone_rust_servers` 変数に集約した。新しい standalone サーバーが追加された場合、この変数に追加するだけで全レシピに反映される。

```just
# standalone ワークスペースを持たない Rust サーバーのパス一覧
_standalone_rust_servers := "regions/service/order/server/rust/order regions/service/inventory/server/rust/inventory regions/service/payment/server/rust/payment regions/business/accounting/server/rust/domain-master"
```

### --if-present オプションの追加（lint-ts / typecheck）

`npm run lint` および `npm run typecheck` の実行に `--if-present` オプションを追加した。

**理由**: モノリポ内の全 TypeScript パッケージが `lint` / `typecheck` スクリプトを `package.json` に定義しているとは限らない。一部のパッケージ（ユーティリティライブラリや設定パッケージ等）はこれらのスクリプトを持たず、`--if-present` なしでは `npm run lint` がエラー終了し、他のパッケージの lint が実行されなくなる。`--if-present` により、スクリプトが存在しないパッケージはスキップし、存在するパッケージのみ実行する。

> **注意**: `test` / `build` スクリプトについては `--if-present` を使用しない方針である（「npm スクリプト実行の --if-present 削除方針」を参照）。`lint` / `typecheck` は補助的なチェックであるため許容するが、`test` / `build` はすべてのパッケージに必須とする。

```just
# TypeScript lint（--if-present: 全パッケージが lint スクリプトを持つとは限らないため）
lint-ts:
    npm run lint --if-present --workspaces
    npm run typecheck --if-present --workspaces
```

### Cargo.toml 検索深度の制限

justfile 内の `find` コマンドによる `Cargo.toml` 探索に `-maxdepth 4` を設定した。モノリポのディレクトリ階層が深いため、無制限の探索ではビルドに無関係な深い階層（`target/` 配下等）の `Cargo.toml` を誤検出するリスクがあった。`-maxdepth 4` により、`regions/{tier}/server/rust/` レベルまでの探索に限定し、検索速度の向上と誤検出の防止を実現する。

```bash
# 変更前: find regions -name Cargo.toml
# 変更後: find regions -maxdepth 4 -name Cargo.toml
```

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
