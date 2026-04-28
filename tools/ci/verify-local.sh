#!/usr/bin/env bash
# =============================================================================
# tools/ci/verify-local.sh
#
# CI（.github/workflows/_reusable-lint.yml + _reusable-precommit.yml）と同等の
# 検査をローカルで一括実行する orchestrator。push 前に走らせて失敗を先取りする
# のが用途。Makefile からは `make verify` / `make verify-quick` で呼ばれる。
#
# モード:
#   full   : 全 tier / 全言語の検査（既定）
#   quick  : git diff origin/<base>..HEAD で変更があったスコープのみ
#
# 環境変数:
#   K1S0_VERIFY_BASE      quick モードの差分起点（既定: origin/main）
#   K1S0_VERIFY_SKIP      "rust,go,dotnet,node,proto,codegen,precommit" のうち
#                         skip したいものを CSV で指定（toolchain 未導入時の逃げ）
#
# 終了コード:
#   0 = 全 check pass / 1 = 1 件以上 fail / 2 = 引数エラー
# =============================================================================
set -uo pipefail

mode="${1:-full}"
case "$mode" in
  full|quick) ;;
  -h|--help)
    sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
    ;;
  *)
    echo "usage: $0 [full|quick]" >&2
    exit 2
    ;;
esac

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root" || { echo "fatal: cd to repo root '$repo_root' が失敗" >&2; exit 1; }

# -----------------------------------------------------------------------------
# 共通ユーティリティ
# -----------------------------------------------------------------------------
declare -a results=()
fail=0
skip_csv="${K1S0_VERIFY_SKIP:-}"

is_skipped() {
  local key="$1"
  [[ ",${skip_csv}," == *",${key},"* ]]
}

run() {
  # run <ラベル> <スキップ判定キー> <コマンド...>
  local label="$1" key="$2"; shift 2
  if is_skipped "$key"; then
    printf '\n=== %s ===\n' "$label"
    echo "SKIP (K1S0_VERIFY_SKIP=$skip_csv)"
    results+=("SKIP  $label")
    return 0
  fi
  printf '\n=== %s ===\n' "$label"
  if "$@"; then
    results+=("PASS  $label")
  else
    local rc=$?
    results+=("FAIL  $label (exit=$rc)")
    fail=1
  fi
}

have() { command -v "$1" >/dev/null 2>&1; }

# -----------------------------------------------------------------------------
# quick モード: 変更スコープ検出
# -----------------------------------------------------------------------------
declare -A scope=(
  [rust]=1 [go]=1 [dotnet]=1 [node]=1
  [proto]=1 [docs]=1 [codegen]=1
)

if [[ "$mode" == "quick" ]]; then
  base="${K1S0_VERIFY_BASE:-origin/main}"
  if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
    echo "warn: base ref '$base' が見つからない。full モードに fallback。" >&2
    mode="full"
  else
    # 全 false にしてから変更パスで true 化
    for k in "${!scope[@]}"; do scope[$k]=0; done
    changed="$(git diff --name-only "${base}...HEAD"; git diff --name-only; git diff --cached --name-only)"
    while IFS= read -r f; do
      [[ -z "$f" ]] && continue
      case "$f" in
        src/contracts/*|tools/codegen/*|buf.gen*.yaml|buf.yaml|buf.work.yaml)
          scope[proto]=1; scope[codegen]=1
          # 契約変更は全 SDK / tier1 を波及（path-filter.yaml 準拠）
          scope[rust]=1; scope[go]=1; scope[dotnet]=1; scope[node]=1
          ;;
        *.rs|*/Cargo.toml|*/Cargo.lock) scope[rust]=1 ;;
        *.go|*/go.mod|*/go.sum)         scope[go]=1 ;;
        *.cs|*.csproj|*.sln)            scope[dotnet]=1 ;;
        *.ts|*.tsx|*.js|*.jsx|*/package.json|*/pnpm-lock.yaml) scope[node]=1 ;;
        *.proto)                        scope[proto]=1; scope[codegen]=1 ;;
        *.md|docs/*)                    scope[docs]=1 ;;
      esac
    done <<< "$changed"
    echo "quick モード: 変更スコープ ="
    for k in "${!scope[@]}"; do echo "  $k=${scope[$k]}"; done
  fi
fi

active() { [[ "${scope[$1]:-1}" == "1" ]]; }

# -----------------------------------------------------------------------------
# 1. pre-commit（全 hook を all-files で）
# -----------------------------------------------------------------------------
if have pre-commit; then
  run "pre-commit run --all-files" precommit pre-commit run --all-files --show-diff-on-failure
else
  results+=("SKIP  pre-commit (未インストール — pip install pre-commit)")
fi

# -----------------------------------------------------------------------------
# 2. codegen / openapi / grpc-docs 差分
#    Makefile 経由ではなく直接スクリプトを呼ぶ。make 未インストールでも動かす
#    ことを優先する。Makefile の codegen-check は同じスクリプトを呼ぶラッパなので
#    挙動は等価。
# -----------------------------------------------------------------------------
if active codegen; then
  run "codegen check"   codegen ./tools/codegen/buf/run.sh      --check
  run "openapi check"   codegen ./tools/codegen/openapi/run.sh   --check
  run "grpc-docs check" codegen ./tools/codegen/grpc-docs/run.sh --check
fi

# -----------------------------------------------------------------------------
# 3. proto: buf lint / format / breaking
# -----------------------------------------------------------------------------
if active proto && have buf; then
  # buf v2 workspace は src/contracts/buf.yaml に配置されているため、
  # 全 buf サブコマンドに path を明示する。省略時はリポジトリ root を
  # v1beta1 単一モジュールとして扱い、モジュール内 import の解決に失敗する。
  run "buf lint"   proto buf lint   src/contracts
  run "buf format" proto buf format -d --exit-code src/contracts
  base_ref="${K1S0_VERIFY_BASE:-origin/main}"
  if git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    # against 側にも subdir=src/contracts が必要。省略時はリポジトリ root を
    # 単一モジュール扱いし、モジュール内 import を解決できず fail する。
    run "buf breaking (vs $base_ref)" proto \
      buf breaking src/contracts \
        --against ".git#branch=${base_ref##*/},subdir=src/contracts"
  fi
elif active proto; then
  results+=("SKIP  buf (未インストール)")
fi

# -----------------------------------------------------------------------------
# 4. Rust workspaces / 単独 crate
#    workspace 配下の member は親 workspace から一括で fmt/clippy するため、
#    「祖先に Cargo.toml が無い Cargo.toml」だけを root として走査する。
# -----------------------------------------------------------------------------
is_cargo_root() {
  # 入力を絶対パス化してから祖先を遡る（find の "./..." と repo_root の絶対パスが
  # 一致せず無限ループに陥るのを防ぐ）。
  local abs d prev
  abs="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
  d="$(dirname "$(dirname "$abs")")"
  while [[ "$d" != "$repo_root" && "$d" != "/" && -n "$d" ]]; do
    [[ -f "$d/Cargo.toml" ]] && return 1
    prev="$d"
    d="$(dirname "$d")"
    [[ "$d" == "$prev" ]] && break
  done
  return 0
}

if active rust && have cargo; then
  while IFS= read -r cargo_toml; do
    is_cargo_root "$cargo_toml" || continue
    dir="$(dirname "$cargo_toml")"
    run "cargo fmt --check ($dir)" rust \
      bash -c "cd '$dir' && cargo fmt --all -- --check"
    run "cargo clippy ($dir)"      rust \
      bash -c "cd '$dir' && cargo clippy --workspace --all-targets -- -D warnings"
  done < <(
    # tests/fuzz/rust は cargo-fuzz crate（libfuzzer-sys 依存・nightly 想定）。
    # 通常 CI の lint 対象外で、cron 週次（tests/fuzz/README.md）で実行する。
    # ローカル verify から除外し、CI と境界を揃える。
    find "$repo_root" -maxdepth 5 -name Cargo.toml \
      -not -path "*/target/*" \
      -not -path "*/third_party/*" \
      -not -path "*/node_modules/*" \
      -not -path "*/tests/fuzz/*"
  )
elif active rust; then
  results+=("SKIP  cargo (未インストール)")
fi

# -----------------------------------------------------------------------------
# 5. Go modules
# -----------------------------------------------------------------------------
if active go && have go; then
  while IFS= read -r mod; do
    dir="$(dirname "$mod")"
    run "go vet ($dir)" go bash -c "cd '$dir' && go vet ./..."
    if have golangci-lint; then
      run "golangci-lint ($dir)" go bash -c "cd '$dir' && golangci-lint run --timeout=5m"
    fi
  done < <(
    find . -maxdepth 5 -name go.mod \
      -not -path "*/node_modules/*" -not -path "*/third_party/*"
  )
elif active go; then
  results+=("SKIP  go (未インストール)")
fi

# -----------------------------------------------------------------------------
# 6. .NET projects
# -----------------------------------------------------------------------------
if active dotnet && have dotnet; then
  while IFS= read -r proj; do
    run "dotnet format ($proj)" dotnet \
      dotnet format "$proj" --verify-no-changes --severity warn
  done < <(
    find . -maxdepth 6 -name "*.csproj" \
      -not -path "*/bin/*" -not -path "*/obj/*" -not -path "*/third_party/*"
  )
elif active dotnet; then
  results+=("SKIP  dotnet (未インストール)")
fi

# -----------------------------------------------------------------------------
# 7. pnpm packages
# -----------------------------------------------------------------------------
if active node && have pnpm; then
  while IFS= read -r pkg; do
    dir="$(dirname "$pkg")"
    run "pnpm install --frozen-lockfile ($dir)" node \
      bash -c "cd '$dir' && pnpm install --frozen-lockfile"
    if grep -q '"lint"' "$pkg"; then
      run "pnpm run lint ($dir)" node bash -c "cd '$dir' && pnpm run lint"
    fi
    if grep -q '"typecheck"' "$pkg"; then
      run "pnpm run typecheck ($dir)" node bash -c "cd '$dir' && pnpm run typecheck"
    fi
  done < <(
    find . -maxdepth 5 -name package.json \
      -not -path "*/node_modules/*" -not -path "*/third_party/*"
  )
elif active node; then
  results+=("SKIP  pnpm (未インストール)")
fi

# -----------------------------------------------------------------------------
# サマリ
# -----------------------------------------------------------------------------
printf '\n========== verify summary (%s mode) ==========\n' "$mode"
printf '%s\n' "${results[@]}"
echo "==============================================="

if [[ $fail -ne 0 ]]; then
  echo "✗ 1 件以上の check が失敗しました。push 前に修正してください。"
  exit 1
fi
echo "✓ すべての check が pass しました。"
