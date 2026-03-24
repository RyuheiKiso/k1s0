# k1s0 monorepo build orchestration
# Usage: just <recipe> or just <recipe>-<lang>
# TypeScript レシピは pnpm を使用する（pnpm-workspace.yaml で workspace:* 依存を管理）
# 前提: pnpm がインストール済みであること（npm install -g pnpm または corepack enable pnpm）

set shell := ["bash", "-euo", "pipefail", "-c"]
# Windows 環境では PowerShell を使用する（ただし WSL2/Git Bash 推奨）
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# ローカル開発で起動する Docker Compose profile（infra + system tier）
_dc_profiles := "--profile infra --profile system"

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

# 全サービスの Docker イメージをローカルビルド
docker-build:
    #!/usr/bin/env bash
    set -euo pipefail
    # Rust サーバー（system tier）のイメージビルド
    for dockerfile in $(find regions/system/server/rust -name 'Dockerfile' | sort); do
        server_name="$(basename "$(dirname "$dockerfile")")"
        echo "=== docker build $server_name ==="
        if [ "$server_name" = "graphql-gateway" ]; then
            docker build -f "$dockerfile" -t "k1s0-$server_name" .
        else
            docker build -f "$dockerfile" -t "k1s0-$server_name" regions/system
        fi
    done
    # Go サーバー（bff-proxy）のイメージビルド
    echo "=== docker build bff-proxy ==="
    docker build -t k1s0-bff-proxy regions/system/server/go/bff-proxy

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

# 認証バイパス付きでローカル開発環境を起動（ローカル開発専用・本番では使用不可）
# C-01監査対応: Docker Compose 起動後、Keycloak から RSA 公開鍵を取得して Kong に設定する。
# setup-kong-jwt.sh が Keycloak の起動を最大60秒待機し、kong.dev.yaml のプレースホルダーを置換する。
# 置換後は Kong を再起動して設定を反映させる。
local-up-dev: _check-env
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Starting local dev environment (auth bypass enabled) ==="
    docker compose -f docker-compose.yaml -f docker-compose.dev.yaml {{_dc_profiles}} up -d
    echo "=== [C-01] Setting up Kong JWT RSA public key (waiting for Keycloak...) ==="
    if bash infra/kong/setup-kong-jwt.sh; then
        echo "=== Restarting Kong to apply new RSA public key configuration ==="
        docker compose -f docker-compose.yaml -f docker-compose.dev.yaml restart kong
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
    docker compose --profile {{profile}} up -d

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
security: security-go security-rust security-ts security-dart

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
