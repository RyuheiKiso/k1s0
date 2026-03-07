# k1s0 monorepo build orchestration
# Usage: just <recipe> or just <recipe>-<lang>

set shell := ["bash", "-euo", "pipefail", "-c"]

# デフォルト: ヘルプ表示
default:
    @just --list

# --- モジュール探索ヘルパー（CI と同一パターン） ---

_rust-skip := "CLI/Cargo.toml|regions/system/Cargo.toml|CLI/crates/k1s0-gui/Cargo.toml"

# --- Lint ---

# 全言語リント
lint: lint-go lint-rust lint-ts lint-dart

# Go リント
lint-go:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t modules < <(rg --files -g 'go.mod' regions CLI | sort)
    for mod in "${modules[@]}"; do
        dir="$(dirname "$mod")"
        echo "=== Linting $dir ==="
        (cd "$dir" && golangci-lint run ./... && go vet ./...)
    done

# Rust リント (fmt + clippy)
lint-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t manifests < <(rg --files -g 'Cargo.toml' regions CLI | sort)
    for manifest in "${manifests[@]}"; do
        case "$manifest" in
            CLI/Cargo.toml|regions/system/Cargo.toml|CLI/crates/k1s0-gui/Cargo.toml) continue ;;
        esac
        echo "=== lint $(dirname "$manifest") ==="
        cargo fmt --manifest-path "$manifest" --all -- --check
        cargo clippy --manifest-path "$manifest" --all-targets -- -D warnings
    done

# TypeScript リント
lint-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'package.json' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Linting $dir ==="
        (cd "$dir" && { [ -f package-lock.json ] && npm ci || npm install --no-package-lock; } && npm run lint --if-present && npm run typecheck --if-present)
    done

# Dart リント
lint-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'pubspec.yaml' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Linting $dir ==="
        if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
            (cd "$dir" && flutter pub get && flutter analyze)
        else
            (cd "$dir" && dart pub get && dart analyze)
        fi
    done

# --- Test ---

# 全言語テスト
test: test-go test-rust test-ts test-dart

# Go テスト
test-go:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t modules < <(rg --files -g 'go.mod' regions CLI | sort)
    for mod in "${modules[@]}"; do
        dir="$(dirname "$mod")"
        echo "=== Testing $dir ==="
        (cd "$dir" && go test ./... -race -count=1)
    done

# Rust テスト
test-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t manifests < <(rg --files -g 'Cargo.toml' regions CLI | sort)
    for manifest in "${manifests[@]}"; do
        case "$manifest" in
            CLI/Cargo.toml|regions/system/Cargo.toml|CLI/crates/k1s0-gui/Cargo.toml) continue ;;
        esac
        echo "=== Testing $(dirname "$manifest") ==="
        cargo test --manifest-path "$manifest" --all
    done

# TypeScript テスト
test-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'package.json' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Testing $dir ==="
        (cd "$dir" && { [ -f package-lock.json ] && npm ci || npm install --no-package-lock; } && npm test --if-present)
    done

# Dart テスト
test-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'pubspec.yaml' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Testing $dir ==="
        if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
            (cd "$dir" && flutter pub get && flutter test)
        else
            (cd "$dir" && dart pub get && dart test)
        fi
    done

# --- Format ---

# 全言語フォーマット
fmt: fmt-go fmt-rust fmt-ts fmt-dart

# Go フォーマット
fmt-go:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t modules < <(rg --files -g 'go.mod' regions CLI | sort)
    for mod in "${modules[@]}"; do
        dir="$(dirname "$mod")"
        echo "=== Formatting $dir ==="
        (cd "$dir" && gofmt -w .)
    done

# Rust フォーマット
fmt-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t manifests < <(rg --files -g 'Cargo.toml' regions CLI | sort)
    for manifest in "${manifests[@]}"; do
        case "$manifest" in
            CLI/Cargo.toml|regions/system/Cargo.toml|CLI/crates/k1s0-gui/Cargo.toml) continue ;;
        esac
        echo "=== Formatting $(dirname "$manifest") ==="
        cargo fmt --manifest-path "$manifest" --all
    done

# TypeScript フォーマット
fmt-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'package.json' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Formatting $dir ==="
        (cd "$dir" && { [ -f package-lock.json ] && npm ci || npm install --no-package-lock; } && npm run format --if-present)
    done

# Dart フォーマット
fmt-dart:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'pubspec.yaml' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Formatting $dir ==="
        if grep -q "sdk: flutter" "$dir/pubspec.yaml"; then
            (cd "$dir" && dart format lib/ test/)
        else
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
    mapfile -t modules < <(rg --files -g 'go.mod' regions CLI | sort)
    for mod in "${modules[@]}"; do
        dir="$(dirname "$mod")"
        echo "=== Building $dir ==="
        (cd "$dir" && go build ./...)
    done

# Rust ビルド
build-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t manifests < <(rg --files -g 'Cargo.toml' regions CLI | sort)
    for manifest in "${manifests[@]}"; do
        case "$manifest" in
            CLI/Cargo.toml|regions/system/Cargo.toml|CLI/crates/k1s0-gui/Cargo.toml) continue ;;
        esac
        echo "=== Building $(dirname "$manifest") ==="
        cargo build --manifest-path "$manifest" --all-targets
    done

# TypeScript ビルド
build-ts:
    #!/usr/bin/env bash
    set -euo pipefail
    mapfile -t packages < <(rg --files -g 'package.json' regions CLI | sort | xargs -n1 dirname)
    for dir in "${packages[@]}"; do
        echo "=== Building $dir ==="
        (cd "$dir" && { [ -f package-lock.json ] && npm ci || npm install --no-package-lock; } && npm run build --if-present)
    done

# --- Proto ---

# Proto コード生成
proto:
    ./scripts/generate-proto.sh

# Client SDK 生成
gen-sdk service proto="api/proto":
    ./scripts/generate-client-sdk.sh --service {{service}} --proto {{proto}}

# --- CI ---

# CI 全実行（lint + test + build）
ci: lint test build
