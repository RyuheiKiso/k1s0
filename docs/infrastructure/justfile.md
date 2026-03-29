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
| Node.js + pnpm | 22 + 9.x | TypeScript ビルド・テスト（`pnpm-workspace.yaml` の `workspace:*` 依存解決に pnpm が必要） |
| Flutter | 3.24 | Dart/Flutter ビルド・テスト |

### pnpm のインストール

TypeScript レシピは `npm ci` の代わりに `pnpm install --frozen-lockfile` を使用する。`pnpm` は以下のいずれかの方法でインストールする:

```bash
# Node.js 付属の corepack を有効化（推奨）
corepack enable pnpm

# または npm でグローバルインストール
npm install -g pnpm
```

### Windows 対応

justfile には以下の設定が含まれる:

```just
set shell := ["bash", "-euo", "pipefail", "-c"]
# Windows 環境では PowerShell を使用する（ただし WSL2/Git Bash 推奨）
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
```

- **推奨環境:** WSL2 上の Ubuntu/Debian
- **Git Bash:** 動作するが `cygpath` 依存の一部スクリプトで制限あり
- **PowerShell ネイティブ:** `set windows-shell` により just からのレシピ実行は可能だが、bash スクリプト（`#!/usr/bin/env bash` shebang）は WSL2 または Git Bash が必要
- **PowerShell 直接実行:** 非対応（`_check-env` レシピで検出・エラー終了）

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
| `lint-ts` | TypeScript: `pnpm run lint` + `pnpm run typecheck` |
| `lint-dart` | Dart/Flutter: `dart analyze` / `flutter analyze` |
| `test` | 全言語テスト (`test-go` + `test-rust` + `test-ts` + `test-dart`) |
| `test-go` | Go: `go test ./... -race -count=1` |
| `test-rust` | Rust: `cargo test --all` |
| `test-ts` | TypeScript: `pnpm test` |
| `test-dart` | Dart/Flutter: `dart test` / `flutter test` |
| `fmt` | 全言語フォーマット (`fmt-go` + `fmt-rust` + `fmt-ts` + `fmt-dart`) |
| `fmt-go` | Go: `gofmt -w .` |
| `fmt-rust` | Rust: `cargo fmt --all` |
| `fmt-ts` | TypeScript: `pnpm run format` |
| `fmt-dart` | Dart: `dart format lib/ test/` |
| `build` | 全言語ビルド (`build-go` + `build-rust` + `build-ts` + `build-dart`) |
| `build-go` | Go: `go build ./...` |
| `build-rust` | Rust: `cargo build --all-targets` |
| `build-ts` | TypeScript: `pnpm run build` |
| `build-dart` | Dart/Flutter: `flutter build web` / `dart analyze`（ライブラリ） |
| `docker-build` | 全 Docker イメージのローカルビルド（最大 4 並列）。環境変数 `CARGO_FEATURES` を設定すると `--build-arg CARGO_FEATURES=...` 経由で各 Dockerfile に渡される（HIGH-3 監査対応）。例: `CARGO_FEATURES=k1s0-server-common/dev-auth-bypass just docker-build` |
| `docker-build-safe` | **OOM 防止**のための安全なビルド（`--parallel 2` に制限）。WSL2 や Docker Desktop でメモリ不足が発生する場合に使用（HIGH-2 監査対応） |
| `migrate path` | 指定パスの DB マイグレーションを実行（`sqlx migrate run`） |
| `migrate-all` | **全システム DB のマイグレーション一括実行**（初回セットアップ用）。`just local-up-profile infra` の後に実行する（HIGH-3/HIGH-4 監査対応） |
| `migrate-all-docker` | **sqlx-cli 未インストール環境向け Docker 経由マイグレーション**（C-03 監査対応）。`docker compose exec postgres` 経由で `*_up.sql` を適用する。Windows Git Bash 等で sqlx-cli が使えない場合に使用する |
| `proto` | Proto コード生成 (`scripts/generate-proto.sh`) |
| `ci` | CI 全実行 (`lint` + `test` + `build`) |
| `security` | 全言語セキュリティスキャン (`security-go` + `security-rust` + `security-ts` + `security-dart`) |
| `security-go` | Go: `govulncheck ./...`（全モジュール自動探索） |
| `security-rust` | Rust: `cargo audit`（CLI/ + regions/system/ ワークスペース） |
| `security-ts` | TypeScript: `pnpm audit --audit-level=high`（全 pnpm-lock.yaml 自動探索） |
| `security-dart` | Dart/Flutter: `pub outdated`（advisory、全 pubspec.yaml 自動探索） |

## 技術監査対応の改善事項

### graphql-gateway Docker ビルドの例外処理（MED-03 / LOW-04 対応）

`docker-build` レシピは全 Rust サービスを `context: regions/system` でビルドするが、`graphql-gateway` のみリポジトリルート（`.`）をビルドコンテキストとする例外がある。

| サービス | ビルドコンテキスト | 理由 |
| --- | --- | --- |
| graphql-gateway 以外の全 Rust サービス | `regions/system` | サービス範囲内のソースのみで完結する |
| `graphql-gateway` | `.`（リポジトリルート） | `tonic-build` が `api/proto` ディレクトリを参照するため |

```bash
# graphql-gateway は特別なビルドコンテキストを使用
docker build -f regions/system/server/rust/graphql-gateway/Dockerfile \
  -t k1s0-graphql-gateway .

# その他の全 Rust サービスは regions/system をコンテキストとする
docker build -f regions/system/server/rust/auth/Dockerfile \
  -t k1s0-auth regions/system
```

`docker-compose.yaml` では `graphql-gateway-rust` サービスの `build.context` が `'.'` に設定されており、この挙動と一致している。詳細は `regions/system/server/rust/graphql-gateway/Dockerfile` の冒頭コメントを参照。

### `local-up` コマンドの修正（C-02 対応）

`just local-up` が `docker-compose.dev.yaml` を使用せず、`KEYCLOAK_ADMIN_PASSWORD` 等の必須環境変数が未定義でエラーになる問題を修正した。

| コマンド | 説明 |
|---------|------|
| `just local-up` | `local-up-dev` の別名。開発環境の標準起動コマンド。 |
| `just local-up-dev` | `docker-compose.yaml` + `docker-compose.dev.yaml` を使用。必須変数のデフォルト値を自動提供。 |
| `just local-up-base` | `docker-compose.yaml` 単体起動（CI・本番確認用）。`.env` に必須変数を設定した上で使用すること。 |

### Docker Compose プロファイル構成

AI 関連サービス（`ai-gateway-rust`, `ai-agent-rust`）は実験段階のため、`system` プロファイルから分離し `experimental-ai` プロファイルに移動している。

| プロファイル | 用途 | 起動コマンド例 |
| --- | --- | --- |
| `infra` | DB・Kafka・Redis 等のインフラ | `docker compose --profile infra up -d` |
| `system` | システムサービス群（本番相当） | `docker compose --profile infra --profile system up -d` |
| `business` | ビジネス tier サービス群 | `docker compose --profile infra --profile system --profile business up -d` |
| `service` | サービス tier サービス群 | `docker compose --profile infra --profile system --profile service up -d` |
| `observability` | 可観測性スタック（Jaeger 等） | `docker compose --profile observability up -d` |
| `experimental-ai` | AI 関連サービス（実験的） | `docker compose --profile infra --profile system --profile experimental-ai up -d` |

**HIGH-4 監査対応**: `just local-down` で全コンテナを確実に停止するため、`_dc_profiles` に `business`・`service`・`observability` プロファイルを追加した。旧設定では `infra` + `system` のみで、`local-down` 実行後も business/service/observability コンテナが残留していた。

通常の開発・テストでは `experimental-ai` プロファイルは不要。AI 機能の検証が必要な場合のみ追加する。

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
_standalone_rust_servers := "regions/service/task/server/rust/task regions/service/board/server/rust/board regions/service/activity/server/rust/activity regions/business/taskmanagement/server/rust/project-master"
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
- **TypeScript**: 個別モジュール反復（`pnpm install --frozen-lockfile` → `pnpm run <script>` の順で実行。`pnpm-workspace.yaml` の `workspace:*` 依存を正しく解決するため pnpm を使用）
- **Dart**: 個別モジュール反復（ワークスペース機構がないため）

### 新サービス追加時

1. `modules.yaml` にエントリを追加する
2. justfile・CI の両方に自動的に反映される（コード変更不要）
