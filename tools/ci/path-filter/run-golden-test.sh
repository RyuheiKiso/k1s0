#!/usr/bin/env bash
# =============================================================================
# tools/ci/path-filter/run-golden-test.sh — filter 変更回帰テスト
#
# 設計: docs/05_実装/30_CI_CD設計/20_path_filter選択ビルド/01_path_filter選択ビルド.md
# IMP-CI-PF-035: filters.yaml 変更時の golden test による回帰防止
#
# 役割:
#   tools/ci/path-filter/filters.yaml の変更が、過去の典型的な changed-files
#   ケースに対して期待 outputs を変えない（あるいは意図通り変える）ことを保証する。
#
# 入力:
#   tests/ci/path-filter-golden/<id>.json          — 変更ファイル一覧 (.files: [])
#   tests/ci/path-filter-golden/<id>.expected.json — 期待 filter outputs (key: bool)
#
# 出力:
#   stdout に "PASS: <id>" / "FAIL: <id>" を逐次出力
#   全件 pass で exit 0、1 件でも regression があれば exit 1
#
# 依存:
#   python3 (>= 3.8) と PyYAML（pip install pyyaml で導入。Dev Container
#   docs-writer / infra-ops プロファイルに同梱済み想定）
#
# Usage:
#   tools/ci/path-filter/run-golden-test.sh         # 全 golden ケース実行
#   tools/ci/path-filter/run-golden-test.sh --list  # 検出されたケース一覧のみ
#   tools/ci/path-filter/run-golden-test.sh -h      # ヘルプ
# =============================================================================
set -euo pipefail

# --- 引数解析（--help / --list / 通常実行）-----------------------------------
LIST_ONLY=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            sed -n '3,28p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        --list)
            LIST_ONLY=1
            shift
            ;;
        *)
            echo "[error] 未知のオプション: $1" >&2
            exit 2
            ;;
    esac
done

# --- パス解決（リポジトリルート起点）-----------------------------------------
SELF_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SELF_DIR}" rev-parse --show-toplevel 2>/dev/null || (cd "${SELF_DIR}/../../.." && pwd))"
FILTERS="${REPO_ROOT}/tools/ci/path-filter/filters.yaml"
GOLDEN_DIR="${REPO_ROOT}/tests/ci/path-filter-golden"

# 必須ファイル存在チェック
if [[ ! -f "${FILTERS}" ]]; then
    echo "[error] filters.yaml が見つからない: ${FILTERS}" >&2
    exit 2
fi
if [[ ! -d "${GOLDEN_DIR}" ]]; then
    echo "[error] golden dir が見つからない: ${GOLDEN_DIR}" >&2
    exit 2
fi

# python3 の存在チェック
if ! command -v python3 >/dev/null 2>&1; then
    echo "[error] python3 が必要（PyYAML も）" >&2
    exit 2
fi

# --list は実装ラッパで python に委譲（python 側で集計のみ）
export GOLDEN_LIST_ONLY="${LIST_ONLY}"

# --- 本体（python による glob 評価と diff 判定）-------------------------------
# minimatch-like glob（**/* / **/ / * / ?）を pure stdlib で実装する。
# dorny/paths-filter@v3 と完全一致を保証するわけではないが、
# k1s0 の filter で使うパターン範囲では同等動作する。
exec python3 - "${FILTERS}" "${GOLDEN_DIR}" <<'PYEOF'
import json
import os
import re
import sys
from pathlib import Path

# PyYAML 必須（CI 上は Dev Container プロファイルで導入される）
try:
    import yaml
except ImportError:
    print("[error] PyYAML が必要: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

# 引数からパスを受け取る（bash 側で REPO_ROOT 起点で解決済み）
filters_path = Path(sys.argv[1])
golden_dir = Path(sys.argv[2])
list_only = os.environ.get("GOLDEN_LIST_ONLY", "0") == "1"


# --- glob → regex 変換 ----------------------------------------------------
# `**` はディレクトリ階層を任意で吸収、`*` は単一セグメント内のみを吸収する。
# 参考: minimatch / globstar の素朴な実装。
def glob_to_regex(pattern: str) -> re.Pattern:
    out = ["^"]
    i = 0
    while i < len(pattern):
        c = pattern[i]
        # `**/` または `**` の処理
        if c == "*" and i + 1 < len(pattern) and pattern[i + 1] == "*":
            # `**/` : 0 段以上の任意ディレクトリにマッチ
            if i + 2 < len(pattern) and pattern[i + 2] == "/":
                out.append("(?:.*/)?")
                i += 3
                continue
            # 末尾 `**` : 任意（/ 含む）
            out.append(".*")
            i += 2
            continue
        # 単独 `*` : / を跨がないセグメント内任意
        if c == "*":
            out.append("[^/]*")
            i += 1
            continue
        # `?` : 単一の非 / 文字
        if c == "?":
            out.append("[^/]")
            i += 1
            continue
        # 正規表現メタ文字はエスケープ
        if c in r".()+|^$\{}[]":
            out.append("\\" + c)
            i += 1
            continue
        out.append(c)
        i += 1
    out.append("$")
    return re.compile("".join(out))


# --- 1 つの filter（globs リスト）が 1 ファイル群とマッチするか判定 -----------
def match_any(globs, files):
    regexes = [glob_to_regex(g) for g in globs]
    for f in files:
        # changed file パスはリポジトリ相対の前提（GitHub Actions の paths-filter と同等）
        for r in regexes:
            if r.match(f):
                return True
    return False


# --- 全 filter を評価 -----------------------------------------------------
def evaluate(filters_def, changed_files):
    return {name: match_any(globs, changed_files) for name, globs in filters_def.items()}


# --- main ----------------------------------------------------------------
with filters_path.open(encoding="utf-8") as f:
    filters_def = yaml.safe_load(f)

# ".expected.json" でない *.json を golden ケースとして拾う
samples = sorted(p for p in golden_dir.glob("*.json") if not p.name.endswith(".expected.json"))

if list_only:
    for s in samples:
        print(s.stem)
    sys.exit(0)

if not samples:
    print(f"[error] golden sample が 1 件も見つからない: {golden_dir}", file=sys.stderr)
    sys.exit(2)

fail_count = 0
for sample in samples:
    with sample.open(encoding="utf-8") as f:
        data = json.load(f)
    files = data.get("files", [])

    # 期待出力ファイル（同名 + .expected.json）
    expected_path = sample.with_name(sample.stem + ".expected.json")
    if not expected_path.exists():
        print(f"FAIL: {sample.stem} — expected ファイル未配置: {expected_path.name}", file=sys.stderr)
        fail_count += 1
        continue
    with expected_path.open(encoding="utf-8") as f:
        expected = json.load(f)

    actual = evaluate(filters_def, files)
    if actual == expected:
        print(f"PASS: {sample.stem}")
        continue

    # 不一致 — 差分を表示
    print(f"FAIL: {sample.stem}", file=sys.stderr)
    keys = sorted(set(actual.keys()) | set(expected.keys()))
    for key in keys:
        a = actual.get(key)
        e = expected.get(key)
        if a != e:
            print(f"  {key}: actual={a} expected={e}", file=sys.stderr)
    fail_count += 1

if fail_count:
    print(f"\n{fail_count} / {len(samples)} sample(s) failed", file=sys.stderr)
    sys.exit(1)

print(f"\nall {len(samples)} sample(s) passed")
PYEOF
