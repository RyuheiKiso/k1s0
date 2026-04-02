# k1s0 monorepo build orchestration
# Usage: just <recipe> or just <recipe>-<lang>
# TypeScript レシピは pnpm を使用する（pnpm-workspace.yaml で workspace:* 依存を管理）
# 前提: pnpm がインストール済みであること（npm install -g pnpm または corepack enable pnpm）

set shell := ["bash", "-euo", "pipefail", "-c"]
# Windows 環境では PowerShell を使用する（ただし WSL2/Git Bash 推奨）
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# ローカル開発で起動する Docker Compose profile（全 tier + observability）
# HIGH-4 監査対応: business/service/observability を追加して local-down で全コンテナを確実に停止する
_dc_profiles := "--profile infra --profile system --profile business --profile service --profile observability"

# standalone Rust サーバーパスは scripts/list-standalone-servers.sh で modules.yaml から動的取得する

# Windows ネイティブ環境チェック: WSL2/Git Bash 以外の環境では警告を出す
# このレシピはサーバービルド系（local-up/local-down）でのみ呼び出す
_check-env:
    #!/usr/bin/env bash
    set -euo pipefail
    # MSYS (Git Bash) / Cygwin 環境を検出した場合は警告を表示
    if [[ "${OSTYPE:-}" == msys* ]] || [[ "${OSTYPE:-}" == cygwin* ]]; then
        echo "WARNING: Windows ネイティブ環境（MSYS/Cygwin）を検出しました。"
        echo "  WSL2 での実行を強く推奨します（Git Bash は一部制限あり）。"
        echo "  詳細: README.md の「前提条件」セクションを参照してください。"
        echo ""
    fi
    # PowerShell から bash を経由して呼ばれた場合の検出（PSModulePath が設定されている）
    if [[ -n "${PSModulePath:-}" ]] && [[ -z "${WSL_DISTRO_NAME:-}" ]]; then
        echo "WARNING: PowerShell 環境から実行されています。"
        echo "  このレシピは Docker Compose を使用するため WSL2 が必要です。"
        echo "  WSL2 または devcontainer 内で実行してください。"
        echo "  Windows ネイティブで可能な作業: just cli-build / cli-test / cli-lint / cli-fmt"
        exit 1
    fi

# デフォルト: ヘルプ表示
default:
    @just --list

# 開発環境の自己診断を実行する
doctor:
    bash scripts/doctor.sh

# --- Lint ---

# 全言語リント
lint: lint-go lint-rust lint-ts lint-dart

# Go リント
lint-go:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から Go の CI 対象モジュールを取得
    mapfile -t modules < <(scripts/list-modules.sh --lang go --no-skip-ci)
    for dir in "${modules[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/go.mod" ]; then
            echo "=== Linting $dir ==="
            (cd "$dir" && golangci-lint run ./... && go vet ./...)
        fi
    done

# Rust リント (fmt + clippy) — ワークスペース一括実行
lint-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    # regions/system ワークスペース — experimental を除外して一括 fmt + clippy
    echo "=== fmt regions/system ==="
    cargo fmt --all --manifest-path regions/system/Cargo.toml -- --check
    echo "=== clippy regions/system ==="
    # 共通スクリプトで experimental パッケージの除外フラグを取得
    excludes=$(bash scripts/list-experimental-excludes.sh)
    cargo clippy --manifest-path regions/system/Cargo.toml --workspace $excludes --all-targets -- -D warnings
    # CLI ワークスペース — k1s0-gui を除外
    echo "=== fmt CLI ==="
    cargo fmt --all --manifest-path CLI/Cargo.toml -- --check
    echo "=== clippy CLI ==="
    cargo clippy --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets -- -D warnings
    # standalone Rust サーバー（business/service tier）— modules.yaml から動的取得する
    mapfile -t _standalone_dirs < <(bash scripts/list-standalone-servers.sh)
    for dir in "${_standalone_dirs[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            echo "=== fmt $dir ==="
            cargo fmt --all --manifest-path "$dir/Cargo.toml" -- --check
            echo "=== clippy $dir ==="
            cargo clippy --manifest-path "$dir/Cargo.toml" --all-targets -- -D warnings
        fi
    done

# TypeScript リント
lint-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から TypeScript の CI 対象モジュールを取得
    mapfile -t packages < <(scripts/list-modules.sh --lang ts --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/package.json" ]; then
            echo "=== Linting $dir ==="
            # pnpm frozen-lockfile でロックファイルに基づいた依存関係をインストールし、リント・型チェックを実行
            # --if-present: スクリプト未定義のパッケージでもエラーにしない
            (cd "$dir" && pnpm install --frozen-lockfile && pnpm run lint --if-present && pnpm run typecheck --if-present)
        fi
    done

# Dart リント
lint-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から Dart の CI 対象モジュールを取得
    mapfile -t packages < <(scripts/list-modules.sh --lang dart --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/pubspec.yaml" ]; then
            echo "=== Linting $dir ==="
            if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
                (cd "$dir" && flutter pub get && flutter analyze)
            else
                (cd "$dir" && dart pub get && dart analyze)
            fi
        fi
    done

# --- Test ---

# 全言語テスト
test: test-go test-rust test-ts test-dart

# Go テスト
test-go:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から Go の CI 対象モジュールを取得
    mapfile -t modules < <(scripts/list-modules.sh --lang go --no-skip-ci)
    for dir in "${modules[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/go.mod" ]; then
            echo "=== Testing $dir ==="
            (cd "$dir" && go test ./... -race -count=1)
        fi
    done

# Rust テスト — ワークスペース一括実行
test-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    # regions/system ワークスペース一括テスト（experimental を除外）
    echo "=== Testing regions/system ==="
    # 共通スクリプトで experimental パッケージの除外フラグを取得
    excludes=$(bash scripts/list-experimental-excludes.sh)
    cargo test --manifest-path regions/system/Cargo.toml --workspace $excludes --features k1s0-tenant-server/test-utils
    # CLI ワークスペース一括テスト（k1s0-gui を除外）
    echo "=== Testing CLI ==="
    cargo test --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui
    # standalone Rust サーバー（business/service tier）— modules.yaml から動的取得する
    mapfile -t _standalone_dirs < <(bash scripts/list-standalone-servers.sh)
    for dir in "${_standalone_dirs[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            echo "=== Testing $dir ==="
            cargo test --manifest-path "$dir/Cargo.toml"
        fi
    done

# TypeScript テスト
test-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から TypeScript の CI 対象モジュールを取得
    mapfile -t packages < <(scripts/list-modules.sh --lang ts --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/package.json" ]; then
            echo "=== Testing $dir ==="
            # pnpm frozen-lockfile でロックファイルに基づいた依存関係をインストールし、テストを実行
            (cd "$dir" && pnpm install --frozen-lockfile && pnpm test --if-present)
        fi
    done

# Dart テスト
test-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から Dart の CI 対象モジュールを取得
    mapfile -t packages < <(scripts/list-modules.sh --lang dart --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/pubspec.yaml" ]; then
            echo "=== Testing $dir ==="
            if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
                (cd "$dir" && flutter pub get && flutter test)
            else
                (cd "$dir" && dart pub get && dart test)
            fi
        fi
    done

# --- Format ---

# 全言語フォーマット
fmt: fmt-go fmt-rust fmt-ts fmt-dart

# Go フォーマット
fmt-go:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t modules < <(scripts/list-modules.sh --lang go --no-skip-ci)
    for dir in "${modules[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/go.mod" ]; then
            echo "=== Formatting $dir ==="
            (cd "$dir" && gofmt -w .)
        fi
    done

# Rust フォーマット — ワークスペース一括実行
fmt-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Formatting regions/system ==="
    cargo fmt --all --manifest-path regions/system/Cargo.toml
    echo "=== Formatting CLI ==="
    cargo fmt --all --manifest-path CLI/Cargo.toml
    # standalone Rust サーバー（business/service tier）— modules.yaml から動的取得する
    mapfile -t _standalone_dirs < <(bash scripts/list-standalone-servers.sh)
    for dir in "${_standalone_dirs[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            echo "=== Formatting $dir ==="
            cargo fmt --all --manifest-path "$dir/Cargo.toml"
        fi
    done

# TypeScript フォーマット
fmt-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(scripts/list-modules.sh --lang ts --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/package.json" ]; then
            echo "=== Formatting $dir ==="
            # pnpm frozen-lockfile でロックファイルに基づいた依存関係をインストールし、フォーマットを実行
            (cd "$dir" && pnpm install --frozen-lockfile && pnpm run format --if-present)
        fi
    done

# Dart フォーマット
fmt-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(scripts/list-modules.sh --lang dart --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/pubspec.yaml" ]; then
            echo "=== Formatting $dir ==="
            (cd "$dir" && dart format lib/ test/)
        fi
    done

# --- Build ---

# 全言語ビルド
build: build-go build-rust build-ts build-dart

# Go ビルド
build-go:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t modules < <(scripts/list-modules.sh --lang go --no-skip-ci)
    for dir in "${modules[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/go.mod" ]; then
            echo "=== Building $dir ==="
            (cd "$dir" && go build ./...)
        fi
    done

# Rust ビルド — ワークスペース一括実行
build-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Building regions/system ==="
    # 共通スクリプトで experimental パッケージの除外フラグを取得
    excludes=$(bash scripts/list-experimental-excludes.sh)
    cargo build --manifest-path regions/system/Cargo.toml --workspace $excludes --all-targets
    echo "=== Building CLI ==="
    cargo build --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets
    # standalone Rust サーバー（business/service tier）— modules.yaml から動的取得する
    mapfile -t _standalone_dirs < <(bash scripts/list-standalone-servers.sh)
    for dir in "${_standalone_dirs[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            echo "=== Building $dir ==="
            cargo build --manifest-path "$dir/Cargo.toml" --all-targets
        fi
    done

# TypeScript ビルド
build-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(scripts/list-modules.sh --lang ts --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/package.json" ]; then
            echo "=== Building $dir ==="
            # pnpm frozen-lockfile でロックファイルに基づいた依存関係をインストールし、ビルドを実行
            (cd "$dir" && pnpm install --frozen-lockfile && pnpm run build --if-present)
        fi
    done

# Dart ビルド（Flutter アプリは web ターゲット、純粋 Dart は compile exe）
build-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    # modules.yaml から Dart の CI 対象モジュールを取得
    mapfile -t packages < <(scripts/list-modules.sh --lang dart --no-skip-ci)
    for dir in "${packages[@]}"; do
        if [ -d "$dir" ] && [ -f "$dir/pubspec.yaml" ]; then
            echo "=== Building $dir ==="
            if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
                # Flutter アプリは web ターゲットでビルド（CI 環境での検証用）
                (cd "$dir" && flutter pub get && flutter build web --no-pub)
            else
                # 純粋 Dart ライブラリは pub get + analyze でビルド相当の検証を行う
                (cd "$dir" && dart pub get && dart analyze)
            fi
        fi
    done

# --- Proto ---

# Proto コード生成
proto:
    ./scripts/generate-proto.sh

# Client SDK 生成
gen-sdk service proto="api/proto":
    ./scripts/generate-client-sdk.sh --service {{service}} --proto {{proto}}

# --- Docker ---

# 全サービスの Docker イメージをローカルビルド（並列実行）
docker-build:
    #!/usr/bin/env bash
    set -euo pipefail
    # BuildKit を明示的に有効化してレイヤーキャッシュとビルドパフォーマンスを最大化する
    export DOCKER_BUILDKIT=1
    # 最大同時並列ビルド数（Docker デーモンのリソース過負荷を防ぐ）
    MAX_PARALLEL=4
    pids=()
    names=()
    failed=0

    # スロット制限付きでバックグラウンドビルドを起動するヘルパー関数
    # 実行中ジョブが MAX_PARALLEL に達した場合、最古のジョブ完了を待ってからスタートする
    start_build() {
        local name="$1"; shift
        while [ "${#pids[@]}" -ge "$MAX_PARALLEL" ]; do
            if ! wait "${pids[0]}"; then
                echo "ERROR: ${names[0]} のビルドが失敗しました" >&2
                failed=1
            fi
            pids=("${pids[@]:1}")
            names=("${names[@]:1}")
        done
        echo "=== [並列] Starting: $name ==="
        "$@" &
        pids+=($!)
        names+=("$name")
    }

    # HIGH-3 監査対応: CARGO_FEATURES ビルド引数を docker build に渡す
    # dev-auth-bypass 等のフィーチャーを docker-build レシピからも制御できるようにする
    # 本番ビルド時は CARGO_FEATURES を空のままにすること（デフォルト空文字）
    CARGO_FEATURES_ARG="${CARGO_FEATURES:-}"

    # Rust サーバー（system tier）のイメージビルド（並列）
    for dockerfile in $(find regions/system/server/rust -name 'Dockerfile' | sort); do
        server_name="$(basename "$(dirname "$dockerfile")")"
        if [ "$server_name" = "graphql-gateway" ]; then
            # graphql-gateway はリポジトリルートをビルドコンテキストとする例外サービス
            # 理由: build.rs が api/proto/gen/rust/ の生成済み .rs ファイルを参照するため
            # buf/validate.proto は BSR 依存のため protoc では解決できず、生成済みファイルを使用する（CRIT-004）
            # 詳細: regions/system/server/rust/graphql-gateway/Dockerfile の冒頭コメントを参照
            start_build "$server_name" docker build -f "$dockerfile" --build-arg "CARGO_FEATURES=${CARGO_FEATURES_ARG}" -t "k1s0-$server_name" .
        else
            start_build "$server_name" docker build -f "$dockerfile" --build-arg "CARGO_FEATURES=${CARGO_FEATURES_ARG}" -t "k1s0-$server_name" regions/system
        fi
    done
    # Go サーバー（bff-proxy）のイメージビルド
    start_build "bff-proxy" docker build -t k1s0-bff-proxy regions/system/server/go/bff-proxy

    # 残り全ビルドの完了を待機して失敗を集計する
    for i in "${!pids[@]}"; do
        if ! wait "${pids[$i]}"; then
            echo "ERROR: ${names[$i]} のビルドが失敗しました" >&2
            failed=1
        fi
    done
    [ "$failed" -eq 0 ] || { echo "ERROR: ビルドが失敗したサービスがあります" >&2; exit 1; }

# メモリ制限環境向けの安全な Docker ビルド（並列数を 2 に制限して OOM を防止する）
# WSL2 や Docker Desktop でメモリ不足が発生する場合に使用する（HIGH-2 監査対応）
# 標準の docker-build は最大 4 並列だが、docker compose build は並列数制限がなく OOM の原因となる
docker-build-safe: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Safe Docker build (--parallel 2, OOM prevention) ==="
    echo "  tip: 通常ビルドでOOMが発生する場合はこのコマンドを使用してください"
    docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
      {{_dc_profiles}} build --parallel 2
    echo "=== Safe Docker build completed ==="

# ローカル開発環境を起動（docker compose + dev overrides）
# C-02監査対応: docker-compose.dev.yaml を自動的に適用し、必須環境変数（KEYCLOAK_ADMIN_PASSWORD 等）の
# デフォルト値を提供することで、新規開発者が just local-up だけで環境を構築できるよう修正。
local-up: local-up-dev

# CI・本番確認用 docker-compose.yaml 単体起動（dev overrides を適用しない）
# 本番同等の起動確認が必要な場合は .env に必須変数を設定した上でこのコマンドを使用すること。
local-up-base: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Starting local development environment (base, no dev overrides) ==="
    if [ -f docker-compose.yaml ] || [ -f docker-compose.yml ]; then
        docker compose {{_dc_profiles}} up -d
    elif [ -f infra/docker/docker-compose.yaml ] || [ -f infra/docker/docker-compose.yml ]; then
        docker compose -f infra/docker/docker-compose.yaml {{_dc_profiles}} up -d
    else
        echo "docker-compose ファイルが見つかりません。scripts/start-local.sh を使用します。"
        bash scripts/start-local.sh
    fi

# ローカル開発環境を停止
local-down: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Stopping local development environment ==="
    if [ -f docker-compose.yaml ] || [ -f docker-compose.yml ]; then
        docker compose {{_dc_profiles}} down
    elif [ -f infra/docker/docker-compose.yaml ] || [ -f infra/docker/docker-compose.yml ]; then
        docker compose -f infra/docker/docker-compose.yaml {{_dc_profiles}} down
    fi

# MED-06 監査対応: Volume を含めて完全クリーンアップする（DB スキーマ変更後に使用）
# PostgreSQL の Docker Volume が既に存在する場合 init-db スクリプトが再実行されないため、
# スキーマ変更（新規 DB 追加等）を反映させるには Volume を削除して再起動する必要がある。
# 警告: ローカルのデータが全て失われる。本番環境では絶対に使用しないこと。
local-down-clean: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== [MED-06] WARNING: Removing all Docker volumes. Local data will be lost. ==="
    echo "  本番環境での実行は禁止。ローカル開発専用コマンドです。"
    docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml {{_dc_profiles}} down -v
    echo "=== Volume cleanup complete. Run 'just local-up' to restart with fresh data. ==="

# 認証バイパス付きでローカル開発環境を起動（ローカル開発専用・本番では使用不可）
# C-01監査対応: Docker Compose 起動後、Keycloak から RSA 公開鍵を取得して Kong に設定する。
# setup-kong-jwt.sh が Keycloak の起動を最大60秒待機し、kong.dev.yaml のプレースホルダーを置換する。
# 置換後は Kong を再起動して設定を反映させる。
#
# --env-file .env.dev を指定する理由:
#   docker-compose.yaml の ${VAR:?error} 構文はホスト環境変数を参照し、ファイルマージより先に評価される。
#   docker-compose.dev.yaml の environment セクションはコンテナ環境変数であり、上記 interpolation を解決できない。
#   .env.dev を --env-file で渡すことでホスト環境変数として展開し、起動エラーを防ぐ。
local-up-dev: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    # .env.dev の存在確認: ない場合は明確なエラーで停止する（サイレントな起動失敗を防止）
    if [ ! -f ".env.dev" ]; then
        echo "Error: .env.dev が見つかりません。リポジトリルートに .env.dev が必要です。" >&2
        echo "  参照: .env.dev はリポジトリに含まれています。git status で確認してください。" >&2
        exit 1
    fi
    echo "=== Starting local dev environment (auth bypass enabled) ==="
    # RUNTIME-001 監査対応: インフラを先に起動し、postgres が healthy になってからシステムサービスを起動する。
    # これにより、スタレイメージや migrate-all-docker との競合を排除し、
    # sqlx::migrate!() がサービス起動前に必ず完了済みの postgres に対して実行される。
    echo "--- Phase 1: インフラサービス起動 ---"
    # CRIT-002 監査対応: --build フラグを追加してスタレイメージを防止する。
    # --build なしでは既存のローカルイメージが使用され、コード変更が反映されないスタレイメージ問題が発生する。
    docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml --profile infra up -d --build
    echo "--- Phase 1: postgres が healthy になるまで待機 ---"
    PG_CONTAINER="k1s0-postgres-1"
    for i in $(seq 1 60); do
      STATUS=$(docker inspect --format='{{{{.State.Health.Status}}}}' "${PG_CONTAINER}" 2>/dev/null || echo "not_found")
      if [ "${STATUS}" = "healthy" ]; then
        echo "postgres healthy"
        break
      fi
      echo "postgres 起動待機中... ${STATUS} (${i}/60)"
      sleep 3
    done
    echo "=== [DOCKER-002] Vault 初期化（init-vault.sh を自動実行） ==="
    VAULT_CONTAINER="k1s0-vault-1"
    for i in $(seq 1 30); do
      STATUS=$(docker inspect --format='{{{{.State.Health.Status}}}}' "${VAULT_CONTAINER}" 2>/dev/null || echo "not_found")
      if [ "${STATUS}" = "healthy" ]; then
        break
      fi
      echo "Vault 起動待機中... ${STATUS} (${i}/30)"
      sleep 2
    done
    if bash infra/docker/vault/init-vault.sh; then
      echo "=== Vault 初期化完了 ==="
    else
      echo "[WARN] Vault 初期化が失敗しました。手動で 'bash infra/docker/vault/init-vault.sh' を実行してください" >&2
    fi
    echo "--- Phase 1.5: データベースマイグレーション実行 (HIGH-003 監査対応) ---"
    # sqlx-cli がインストール済みの場合はマイグレーションを自動実行する
    # 未インストールの場合は WARN を出力して続行する（just doctor で sqlx の確認を推奨）
    if command -v sqlx &>/dev/null; then
        if just migrate-all; then
            echo "--- Phase 1.5: マイグレーション完了 ---"
        else
            echo "[WARN] マイグレーションが失敗しました。サービス起動に影響する可能性があります。" >&2
            echo "  手動でのマイグレーション: just migrate-all" >&2
        fi
    else
        echo "[WARN] sqlx-cli が未インストールのためマイグレーションをスキップします。" >&2
        echo "  インストール: cargo install sqlx-cli --no-default-features --features postgres" >&2
        echo "  その後手動実行: just migrate-all" >&2
    fi
    echo "--- Phase 2: システム/ビジネス/サービス層の起動 ---"
    # CRIT-002 監査対応: --build フラグでスタレイメージを防止する（Phase 1 と同様）
    docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml --profile system --profile business --profile service --profile observability up -d --build
    echo "=== [C-01] Setting up Kong JWT RSA public key (waiting for Keycloak...) ==="
    if bash infra/kong/setup-kong-jwt.sh; then
        echo "=== Restarting Kong to apply new RSA public key configuration ==="
        docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml restart kong
        echo "=== Kong JWT setup complete ==="
    else
        echo "[WARN] Kong JWT setup failed. Kong may crash with placeholder RSA key." >&2
        echo "  Keycloak が起動したら手動で実行してください: bash infra/kong/setup-kong-jwt.sh" >&2
    fi

# 指定プロファイルのみ起動（例: just local-up-profile infra）
local-up-profile profile: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Starting profile: {{profile}} ==="
    # CRIT-002 監査対応: --build フラグを追加してスタレイメージを防止する
    docker compose --profile {{profile}} up -d --build

# 可観測性スタック（Jaeger / Prometheus / Grafana / Loki）を起動
observability-up: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Starting observability stack ==="
    docker compose --profile observability up -d

# 可観測性スタックを停止
observability-down: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose --profile observability down

# サービスのログを表示する（引数なしで全サービス）
logs service="": _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{service}}" ]; then
        docker compose logs -f {{service}}
    else
        docker compose {{_dc_profiles}} logs -f
    fi

# DBマイグレーションを実行する（引数: migrations/ を含むサービスパス）
migrate path=".":
    #!/usr/bin/env bash
    set -euo pipefail
    db_url="${DATABASE_URL:-postgresql://dev:dev@localhost:5432/k1s0}"
    migrations_dir="{{path}}/migrations"
    if [ ! -d "$migrations_dir" ]; then
        echo "Error: migrations/ ディレクトリが見つかりません: $migrations_dir"
        exit 1
    fi
    echo "=== Running migrations from $migrations_dir ==="
    sqlx migrate run --database-url "$db_url" --source "$migrations_dir"

# 全 DB のマイグレーションを一括実行する（初回セットアップ用）
# インフラサービス（postgres）が起動した後、システム/ビジネス/サービスを起動する前に実行する（HIGH-3/HIGH-4 監査対応）
# AVAIL-002 対応: business/service tier のマイグレーションを k1s0 ユーザーで実行する。
#   system tier は専用 _rw ロール（k1s0_auth_rw 等）で接続するため dev ユーザーで DDL 実行が必要。
#   business/service tier のサービスは k1s0 ユーザーで接続するため、k1s0 がテーブルオーナーで
#   ある必要がある（ALTER TABLE ENABLE ROW LEVEL SECURITY 等を k1s0 ユーザーが実行するため）。
# 実行前提: just local-up-profile infra が完了していること
migrate-all:
    #!/usr/bin/env bash
    set -euo pipefail
    PG_HOST="${PG_HOST:-localhost}"
    PG_PORT="${PG_PORT:-5432}"
    # system tier のマイグレーションは dev（postgres スーパーユーザー相当）で実行する。
    # 専用 _rw ロール（k1s0_auth_rw 等）には DDL 権限がないため dev で DDL を実行し、
    # 99-finalize-grants.sh で DML 権限を付与する設計になっている。
    PG_USER="${PG_USER:-dev}"
    PG_PASS="${PG_PASS:-dev}"
    # business/service tier のマイグレーションは k1s0 ユーザーで実行する（AVAIL-002 対応）。
    # サービス起動時も k1s0 ユーザーで接続するため、k1s0 がテーブルオーナーである必要がある。
    # k1s0 のパスワードは .env.dev の K1S0_DB_PASSWORD（ローカル開発: dev-k1s0-local）。
    K1S0_USER="${K1S0_USER:-k1s0}"
    K1S0_PASS="${K1S0_DB_PASSWORD:-dev-k1s0-local}"
    echo "=== Running all DB migrations (system: $PG_USER, business/service: $K1S0_USER) ==="
    echo "  DB host: $PG_HOST:$PG_PORT"
    failed=0
    # --- system tier: dev ユーザーで実行 ---
    echo "--- [system tier] Running migrations as $PG_USER ---"
    for dir in regions/system/database/*/; do
        if [ -d "$dir/migrations" ]; then
            dir_name=$(basename "$dir")
            # ディレクトリ名から実際のDB名への明示的マッピング
            # 単純な tr '-' '_' では導出できない例外を case で処理する:
            #   dlq-manager-db        → dlq_db                (05-dlq-schema.sql が dlq_db に dlq スキーマを作成)
            #   event-monitor-db      → k1s0_event_monitor   (CRIT-001 監査対応: k1s0_system から専用 DB に分離)
            #   master-maintenance-db → k1s0_master_maintenance (CRIT-001 監査対応: k1s0_system から専用 DB に分離)
            #   saga-db               → k1s0_saga             (HIGH-006 監査対応: saga は k1s0_saga 専用 DB（ADR-0060で移行済み）)
            case "$dir_name" in
                dlq-manager-db)        db_name="dlq_db" ;;
                event-monitor-db)      db_name="k1s0_event_monitor" ;;
                master-maintenance-db) db_name="k1s0_master_maintenance" ;;
                saga-db)               db_name="k1s0_saga" ;;
                *)                     db_name=$(echo "$dir_name" | tr '-' '_') ;;
            esac
            db_url="postgresql://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/${db_name}"
            echo "--- Migrating: $dir_name → $db_name ---"
            if sqlx migrate run --database-url "$db_url" --source "$dir/migrations" 2>&1; then
                echo "  OK: $dir_name → $db_name"
            else
                echo "  FAILED: $dir_name → $db_name" >&2
                failed=1
            fi
        fi
    done
    # --- business tier: k1s0 ユーザーで実行（AVAIL-002 対応）---
    # k1s0_business DB に project_master スキーマが存在し、k1s0 に CONNECT + CREATE 権限が付与済み。
    # （infra/docker/init-db/12-project-master-schema.sql で設定）
    echo "--- [business tier] Running migrations as $K1S0_USER ---"
    for dir in regions/business/*/database/*/; do
        if [ -d "$dir/migrations" ]; then
            domain=$(echo "$dir" | cut -d'/' -f3)
            dir_name=$(basename "$dir")
            # business tier の DB マッピング:
            #   project-master-db → k1s0_business（12-project-master-schema.sql）
            case "$dir_name" in
                project-master-db) db_name="k1s0_business" ;;
                *)                 db_name=$(echo "$dir_name" | tr '-' '_') ;;
            esac
            db_url="postgresql://${K1S0_USER}:${K1S0_PASS}@${PG_HOST}:${PG_PORT}/${db_name}"
            echo "--- Migrating: $domain/$dir_name → $db_name ---"
            if sqlx migrate run --database-url "$db_url" --source "$dir/migrations" 2>&1; then
                echo "  OK: $domain/$dir_name → $db_name"
            else
                echo "  FAILED: $domain/$dir_name → $db_name" >&2
                failed=1
            fi
        fi
    done
    # --- service tier: k1s0 ユーザーで実行（AVAIL-002 対応）---
    # task/board/activity はすべて k1s0_service DB を使用。k1s0 に CONNECT + CREATE 権限が付与済み。
    # （infra/docker/init-db/13〜15-*-schema.sql で設定）
    # 各サービスの _sqlx_migrations は別スキーマ（task_service/board_service/activity_service）に
    # 保存されるため、同一 DB への複数マイグレーション実行は安全に行える。
    echo "--- [service tier] Running migrations as $K1S0_USER ---"
    for dir in regions/service/*/database/*/; do
        if [ -d "$dir/migrations" ]; then
            svc=$(echo "$dir" | cut -d'/' -f3)
            dir_name=$(basename "$dir")
            # service tier の DB マッピング:
            #   postgres → k1s0_service（task/board/activity が共有; 13〜15-*-schema.sql）
            case "$dir_name" in
                postgres) db_name="k1s0_service" ;;
                *)        db_name=$(echo "$dir_name" | tr '-' '_') ;;
            esac
            db_url="postgresql://${K1S0_USER}:${K1S0_PASS}@${PG_HOST}:${PG_PORT}/${db_name}"
            echo "--- Migrating: $svc/$dir_name → $db_name ---"
            if sqlx migrate run --database-url "$db_url" --source "$dir/migrations" 2>&1; then
                echo "  OK: $svc/$dir_name → $db_name"
            else
                echo "  FAILED: $svc/$dir_name → $db_name" >&2
                failed=1
            fi
        fi
    done
    [ "$failed" -eq 0 ] || { echo "ERROR: 一部のマイグレーションが失敗しました" >&2; exit 1; }
    echo "=== All DB migrations completed (system/business/service) ==="

# Windows Git Bash など sqlx-cli 未インストール環境向け Docker 経由マイグレーション（C-03 監査対応）
# sqlx-cli をローカルインストールせずに、実行中の postgres コンテナ経由でマイグレーションを適用する
# 実行前提: just local-up-profile infra が完了していること（postgres コンテナが healthy 状態であること）
#
# ⚠️ 重要な制限事項（RUNTIME-001 監査対応）:
#   このコマンドは raw SQL を実行するため sqlx の _sqlx_migrations 追跡テーブルを更新しない。
#   そのため、このコマンド実行後にサービスを起動すると sqlx::migrate!() がすべてのマイグレーションを
#   「未適用」と判断して再実行を試み、CREATE TRIGGER 等の非べき等 SQL でエラーが発生する。
#
#   使用可能なケース: サービスが起動していない状態で手動 SQL 検証のみ行う場合
#   禁止されるケース: just local-up-dev や just local-up-profile system の前後
#
#   通常のセットアップには just migrate-all（sqlx-cli 必要）を使用すること。
#
# MED-004 監査対応: 起動時自動マイグレーションの対象は限定的である点に注意。
#   自動マイグレーションを実行するのは以下の4サービス（sqlx::migrate!() 実装済み）のみ:
#     saga, workflow, scheduler, master-maintenance
#   残りの19サービスは事前に `just migrate-all` または `just migrate-all-docker` の実行が必要。
#   `just local-up-dev` はこれらの事前マイグレーションを自動実行しないため、
#   初回セットアップ時は Phase 1（infra 起動）完了後に `just migrate-all` を手動実行すること。
migrate-all-docker:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Running all system DB migrations via Docker ==="
    echo "[WARNING] RUNTIME-001: このコマンドは _sqlx_migrations を更新しません。" >&2
    echo "[WARNING] 実行後にサービスを起動すると sqlx::migrate!() との競合が発生します。" >&2
    echo "[WARNING] 通常は 'just migrate-all'（sqlx-cli 必要）を使用してください。" >&2
    for dir in regions/system/database/*/; do
        if [ ! -d "$dir/migrations" ]; then
            continue
        fi
        db_dir=$(basename "$dir")
        # ディレクトリ名から実際のDB名への明示的マッピング（migrate-all と同様のロジック）
        # HIGH-006 監査対応: saga-db は k1s0_saga 専用 DB（ADR-0060で移行済み）。migrate-all と統一する。
        case "$db_dir" in
            dlq-manager-db)        db_name="dlq_db" ;;
            event-monitor-db)      db_name="k1s0_system" ;;
            master-maintenance-db) db_name="k1s0_system" ;;
            saga-db)               db_name="k1s0_saga" ;;
            *)                     db_name=$(echo "$db_dir" | tr '-' '_') ;;
        esac
        echo "--- Migrating via Docker: $db_dir → $db_name ---"
        # 対象 DB の存在確認（存在しない場合はスキップ）
        docker compose exec -T postgres psql \
            -U "${PG_USER:-dev}" \
            -d "$db_name" \
            -c "SELECT 1" > /dev/null 2>&1 || { echo "  SKIP: $db_name not found"; continue; }
        # INFRA-04 監査対応: ファイル名は {N}.up.sql 形式のため *.up.sql パターンを使用する
        # 旧パターン *_up.sql はアンダースコア区切りのファイルのみにマッチし 0件だった
        for f in "$dir/migrations"/*.up.sql; do
            [ -f "$f" ] && docker compose exec -T postgres psql \
                -U "${PG_USER:-dev}" \
                -d "$db_name" < "$f" && echo "  Applied: $(basename $f)"
        done
    done
    echo "=== Docker migrations complete ==="

# 指定パスのサービスをリントする（言語を自動検出）
lint-service path:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Linting: {{path}} ==="
    if [ -f "{{path}}/Cargo.toml" ]; then
        cargo fmt --manifest-path "{{path}}/Cargo.toml" --all -- --check
        cargo clippy --manifest-path "{{path}}/Cargo.toml" --all-targets -- -D warnings
    elif [ -f "{{path}}/go.mod" ]; then
        (cd "{{path}}" && golangci-lint run ./... && go vet ./...)
    elif [ -f "{{path}}/package.json" ]; then
        (cd "{{path}}" && pnpm install --frozen-lockfile && pnpm run lint --if-present)
    elif [ -f "{{path}}/pubspec.yaml" ]; then
        (cd "{{path}}" && dart pub get && dart analyze)
    else
        echo "Error: 対応する言語ファイルが見つかりません: {{path}}"
        exit 1
    fi

# 指定パスのサービスをテストする（言語を自動検出）
test-service path:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Testing: {{path}} ==="
    if [ -f "{{path}}/Cargo.toml" ]; then
        cargo test --manifest-path "{{path}}/Cargo.toml"
    elif [ -f "{{path}}/go.mod" ]; then
        (cd "{{path}}" && go test ./... -race -count=1)
    elif [ -f "{{path}}/package.json" ]; then
        (cd "{{path}}" && pnpm install --frozen-lockfile && pnpm test --if-present)
    elif [ -f "{{path}}/pubspec.yaml" ]; then
        (cd "{{path}}" && dart pub get && dart test)
    else
        echo "Error: 対応する言語ファイルが見つかりません: {{path}}"
        exit 1
    fi

# 指定パスのサービスをビルドする（言語を自動検出）
build-service path:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Building: {{path}} ==="
    if [ -f "{{path}}/Cargo.toml" ]; then
        cargo build --manifest-path "{{path}}/Cargo.toml" --all-targets
    elif [ -f "{{path}}/go.mod" ]; then
        (cd "{{path}}" && go build ./...)
    elif [ -f "{{path}}/package.json" ]; then
        (cd "{{path}}" && pnpm install --frozen-lockfile && pnpm run build --if-present)
    elif [ -f "{{path}}/pubspec.yaml" ]; then
        (cd "{{path}}" && dart pub get && dart compile exe .)
    else
        echo "Error: 対応する言語ファイルが見つかりません: {{path}}"
        exit 1
    fi

# 統合テストを実行
integration-test:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Running integration tests ==="
    # 統合テスト対象サーバーを検出して実行
    chmod +x scripts/ci-list-integration-servers.sh
    mapfile -t servers < <(scripts/ci-list-integration-servers.sh)
    for server in "${servers[@]}"; do
        echo "=== Integration test: $server ==="
        if [ -f "$server/Cargo.toml" ]; then
            cargo test --manifest-path "$server/Cargo.toml" --test '*' -- --ignored
        elif [ -f "$server/go.mod" ]; then
            (cd "$server" && go test -tags=integration ./... -race -count=1)
        fi
    done

# --- CI ---

# CI 全実行（lint + test + build）
ci: lint test build

# --- Security ---

# 全言語セキュリティスキャン
security: security-go security-rust security-ts security-dart security-infra

# プレースホルダー値が残っていないか検証する（CI/CD デプロイ前チェック）
check-secrets:
    bash scripts/check-placeholder-secrets.sh

# インフラセキュリティチェック: プレースホルダーが本番ファイルに残っていないことを確認する（H-4 監査対応）
# etcd 暗号化キー等のプレースホルダーが CI/CD でデプロイされることを防ぐための防護策
security-infra:
    @echo "==> infra セキュリティチェック: プレースホルダー検出..."
    @if grep -r "REPLACE_WITH_" infra/kubernetes/ --include="*.yaml" --include="*.yml" -l 2>/dev/null; then \
        echo "ERROR: infra/kubernetes/ にプレースホルダーが残存しています。デプロイ前に実際の値に置換してください。"; \
        exit 1; \
    fi
    @echo "OK: プレースホルダーは検出されませんでした。"

# Go 脆弱性スキャン
security-go:
    bash scripts/security/go-vulncheck.sh

# Rust 脆弱性監査
security-rust:
    bash scripts/security/cargo-audit.sh

# TypeScript/npm 脆弱性監査
security-ts:
    bash scripts/security/npm-audit.sh

# Dart/Flutter 依存チェック
security-dart:
    bash scripts/security/dart-outdated.sh

# --- CLI（Windows ネイティブ対応）---
# 以下のレシピは Windows ネイティブ（Git Bash / PowerShell）でも動作する
# rdkafka / zen-engine に依存しない CLI ワークスペース（k1s0-cli, k1s0-core）のみを対象とする
# k1s0-gui（Tauri）は WebView2 SDK が必要なため除外する

# CLI ビルド（Windows/Unix 共通: cargo コマンドは PowerShell/bash 両方で同一構文）
cli-build:
    cargo build --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets

# CLI テスト（Windows/Unix 共通: cargo コマンドは PowerShell/bash 両方で同一構文）
cli-test:
    cargo test --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui

# CLI リント（Windows/Unix 共通）
[windows]
cli-lint:
    cargo fmt --all --manifest-path CLI/Cargo.toml -- --check
    cargo clippy --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets -- -D warnings

[unix]
cli-lint:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt --all --manifest-path CLI/Cargo.toml -- --check
    cargo clippy --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets -- -D warnings

# CLI フォーマット（Windows/Unix 共通: cargo コマンドは PowerShell/bash 両方で同一構文）
cli-fmt:
    cargo fmt --all --manifest-path CLI/Cargo.toml
