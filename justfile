# k1s0 monorepo build orchestration
# Usage: just <recipe> or just <recipe>-<lang>

set shell := ["bash", "-euo", "pipefail", "-c"]

# デフォルト: ヘルプ表示
default:
    @just --list

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
    # modules.yaml から experimental Rust モジュールを取得し --exclude に変換
    excludes=""
    while IFS= read -r dir; do
        # Cargo.toml から実際の package name を取得（basename と package name が異なる場合に対応）
        pkg_name=$(grep -m1 '^name' "$dir/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
        excludes="$excludes --exclude $pkg_name"
    done < <(scripts/list-modules.sh --lang rust --status experimental)
    # exclude 対象が workspace に存在するか検証
    ws_packages=$(grep -rh '^name' regions/system/*/Cargo.toml regions/system/*/*/Cargo.toml regions/system/*/*/*/Cargo.toml regions/system/*/*/*/*/Cargo.toml 2>/dev/null | sed 's/.*"\(.*\)"/\1/')
    for exc in $excludes; do
      if [ "$exc" = "--exclude" ]; then continue; fi
      if ! echo "$ws_packages" | grep -qx "$exc"; then
        echo "ERROR: excluded package '$exc' not found in workspace"
        exit 1
      fi
    done
    cargo clippy --manifest-path regions/system/Cargo.toml --workspace $excludes --all-targets -- -D warnings
    # CLI ワークスペース — k1s0-gui を除外
    echo "=== fmt CLI ==="
    cargo fmt --all --manifest-path CLI/Cargo.toml -- --check
    echo "=== clippy CLI ==="
    cargo clippy --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets -- -D warnings
    # standalone Rust サーバー（business/service tier）
    for dir in regions/business/accounting/server/rust/domain-master regions/service/inventory/server/rust/inventory regions/service/order/server/rust/order regions/service/payment/server/rust/payment; do
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
            # package-lock.json を使って依存関係をインストールし、リント・型チェックを実行
            (cd "$dir" && npm ci && npm run lint --if-present && npm run typecheck --if-present)
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
    excludes=""
    while IFS= read -r dir; do
        # Cargo.toml から実際の package name を取得（basename と package name が異なる場合に対応）
        pkg_name=$(grep -m1 '^name' "$dir/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
        excludes="$excludes --exclude $pkg_name"
    done < <(scripts/list-modules.sh --lang rust --status experimental)
    # exclude 対象が workspace に存在するか検証
    ws_packages=$(grep -rh '^name' regions/system/*/Cargo.toml regions/system/*/*/Cargo.toml regions/system/*/*/*/Cargo.toml regions/system/*/*/*/*/Cargo.toml 2>/dev/null | sed 's/.*"\(.*\)"/\1/')
    for exc in $excludes; do
      if [ "$exc" = "--exclude" ]; then continue; fi
      if ! echo "$ws_packages" | grep -qx "$exc"; then
        echo "ERROR: excluded package '$exc' not found in workspace"
        exit 1
      fi
    done
    cargo test --manifest-path regions/system/Cargo.toml --workspace $excludes --features k1s0-tenant-server/test-utils
    # CLI ワークスペース一括テスト（k1s0-gui を除外）
    echo "=== Testing CLI ==="
    cargo test --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui
    # standalone Rust サーバー（business/service tier）
    for dir in regions/business/accounting/server/rust/domain-master regions/service/inventory/server/rust/inventory regions/service/order/server/rust/order regions/service/payment/server/rust/payment; do
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
            # package-lock.json を使って依存関係をインストールし、テストを実行
            (cd "$dir" && npm ci && npm test --if-present)
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
    # standalone Rust サーバー（business/service tier）
    for dir in regions/business/accounting/server/rust/domain-master regions/service/inventory/server/rust/inventory regions/service/order/server/rust/order regions/service/payment/server/rust/payment; do
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
            # package-lock.json を使って依存関係をインストールし、フォーマットを実行
            (cd "$dir" && npm ci && npm run format --if-present)
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
build: build-go build-rust build-ts

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
    excludes=""
    while IFS= read -r dir; do
        # Cargo.toml から実際の package name を取得（basename と package name が異なる場合に対応）
        pkg_name=$(grep -m1 '^name' "$dir/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
        excludes="$excludes --exclude $pkg_name"
    done < <(scripts/list-modules.sh --lang rust --status experimental)
    # exclude 対象が workspace に存在するか検証
    ws_packages=$(grep -rh '^name' regions/system/*/Cargo.toml regions/system/*/*/Cargo.toml regions/system/*/*/*/Cargo.toml regions/system/*/*/*/*/Cargo.toml 2>/dev/null | sed 's/.*"\(.*\)"/\1/')
    for exc in $excludes; do
      if [ "$exc" = "--exclude" ]; then continue; fi
      if ! echo "$ws_packages" | grep -qx "$exc"; then
        echo "ERROR: excluded package '$exc' not found in workspace"
        exit 1
      fi
    done
    cargo build --manifest-path regions/system/Cargo.toml --workspace $excludes --all-targets
    echo "=== Building CLI ==="
    cargo build --manifest-path CLI/Cargo.toml --workspace --exclude k1s0-gui --all-targets
    # standalone Rust サーバー（business/service tier）
    for dir in regions/business/accounting/server/rust/domain-master regions/service/inventory/server/rust/inventory regions/service/order/server/rust/order regions/service/payment/server/rust/payment; do
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
            # package-lock.json を使って依存関係をインストールし、ビルドを実行
            (cd "$dir" && npm ci && npm run build --if-present)
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

# ローカル開発環境を起動（docker compose）
local-up:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Starting local development environment ==="
    if [ -f docker-compose.yaml ] || [ -f docker-compose.yml ]; then
        docker compose up -d
    elif [ -f infra/docker/docker-compose.yaml ] || [ -f infra/docker/docker-compose.yml ]; then
        docker compose -f infra/docker/docker-compose.yaml up -d
    else
        echo "docker-compose ファイルが見つかりません。scripts/start-local.sh を使用します。"
        bash scripts/start-local.sh
    fi

# ローカル開発環境を停止
local-down:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Stopping local development environment ==="
    if [ -f docker-compose.yaml ] || [ -f docker-compose.yml ]; then
        docker compose down
    elif [ -f infra/docker/docker-compose.yaml ] || [ -f infra/docker/docker-compose.yml ]; then
        docker compose -f infra/docker/docker-compose.yaml down
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
