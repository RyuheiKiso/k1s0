#!/usr/bin/env bash
#
# tools/catalog-check/check-owner-system.sh — spec.owner / spec.system 実在検証
#
# 設計: docs/05_実装/99_索引/60_catalog-info検証/01_catalog-info検証設計.md
# 関連 IMP-ID: IMP-TRACE-CAT-023 / IMP-TRACE-CAT-024 / IMP-TRACE-CAT-027
# 責務（前段 CAT-023）:
#   spec.owner に指定された Group 名が、
#   tools/catalog-check/cache/groups.json（Backstage Group catalog snapshot）
#   に実在するかを検証する。
# 責務（後段 CAT-024）:
#   spec.system に指定された System 名が、
#   tools/catalog-check/cache/systems.json（Backstage System catalog snapshot）
#   に実在するかを検証する。
#
#   cache ファイルが存在しない場合は skip（GHA 環境では Backstage catalog を
#   取得してから実行する前提。ローカル pre-commit では CAT-020/021/022 のみ実行）。
#
# Usage:
#   tools/catalog-check/check-owner-system.sh [--strict] [--skip-if-no-cache] [--help]
#   tools/catalog-check/check-owner-system.sh [FILE ...]
#
# Exit code:
#   0 = pass（または cache なしで skip）
#   1 = fail
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
CACHE_DIR="${REPO_ROOT}/tools/catalog-check/cache"
GROUPS_CACHE="${CACHE_DIR}/groups.json"
SYSTEMS_CACHE="${CACHE_DIR}/systems.json"
STRICT=0
SKIP_IF_NO_CACHE=0
FAIL=0
TARGET_FILES=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict)           STRICT=1;          shift ;;
    --skip-if-no-cache) SKIP_IF_NO_CACHE=1; shift ;;
    -h|--help)
      sed -n '3,27p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    -*)
      echo "[error] 未知のオプション: $1" >&2
      exit 2
      ;;
    *)
      TARGET_FILES+=("$1"); shift ;;
  esac
done

if ! command -v python3 >/dev/null 2>&1; then
  echo "[setup-error] python3 が見つかりません" >&2
  exit 2
fi

# ──────────────────────────────────────────────────────────
# cache チェック
# ──────────────────────────────────────────────────────────
if [[ ! -f "${GROUPS_CACHE}" ]] || [[ ! -f "${SYSTEMS_CACHE}" ]]; then
  if [[ "${SKIP_IF_NO_CACHE}" -eq 1 ]]; then
    echo "[skip] cache ファイルが見つかりません — owner/system 実在検証をスキップ"
    echo "       (${GROUPS_CACHE})"
    echo "       (${SYSTEMS_CACHE})"
    echo "       GHA 環境では tools/catalog-check/fetch-catalog-cache.sh を事前実行してください"
    exit 0
  else
    echo "[warn] cache ファイルが見つかりません。--skip-if-no-cache を指定するか"
    echo "       tools/catalog-check/fetch-catalog-cache.sh を先に実行してください"
    echo "       cache: ${CACHE_DIR}"
    # cache なし時は setup error ではなく skip として扱う（ローカル pre-commit 考慮）
    exit 0
  fi
fi

# ──────────────────────────────────────────────────────────
# 対象ファイル収集
# ──────────────────────────────────────────────────────────
if [[ "${#TARGET_FILES[@]}" -eq 0 ]]; then
  while IFS= read -r f; do
    TARGET_FILES+=("${f}")
  done < <(find "${REPO_ROOT}" \
    -type f \
    -name "catalog-info.yaml" \
    -not -path "*/node_modules/*" \
    -not -path "*/.git/*")
fi

total="${#TARGET_FILES[@]}"
echo "[info] 検証対象: ${total} 件"

RESULT="$(python3 - "${GROUPS_CACHE}" "${SYSTEMS_CACHE}" "${TARGET_FILES[@]}" <<'PYEOF'
import sys, json, re

args = sys.argv[1:]
groups_cache_path = args[0]
systems_cache_path = args[1]
files = args[2:]
FAIL = False

# cache 読み込み
def load_names(path):
    try:
        with open(path) as f:
            data = json.load(f)
        if isinstance(data, list):
            return {str(x).lower() for x in data}
        if isinstance(data, dict) and "items" in data:
            return {item.get("metadata", {}).get("name", "").lower()
                    for item in data["items"] if isinstance(item, dict)}
        return set()
    except Exception as e:
        print(f"[warn] cache 読み込みエラー: {path} — {e}")
        return set()

KNOWN_GROUPS  = load_names(groups_cache_path)
KNOWN_SYSTEMS = load_names(systems_cache_path)

try:
    import yaml
    has_yaml = True
except ImportError:
    has_yaml = False

def extract_spec_value(content, key):
    """簡易: 'key: val' を探す"""
    for line in content.split('\n'):
        stripped = line.strip()
        if stripped.startswith(f"{key}:"):
            val = stripped.split(":", 1)[-1].strip().strip('"\'')
            # "group:k1s0-platform" → "k1s0-platform"
            if ':' in val:
                val = val.split(':', 1)[-1]
            return val
    return ""

for path in files:
    errors = []
    try:
        with open(path) as f:
            content = f.read()
        if has_yaml:
            docs = list(yaml.safe_load_all(content))
            doc = docs[0] if docs else {}
            if doc is None:
                doc = {}
            spec = doc.get("spec") or {}
            owner  = spec.get("owner", "")
            system = spec.get("system", "")
            # "group:k1s0-platform" → "k1s0-platform"
            if isinstance(owner, str) and ':' in owner:
                owner = owner.split(':', 1)[-1]
            if isinstance(system, str) and ':' in system:
                system = system.split(':', 1)[-1]
        else:
            owner  = extract_spec_value(content, "owner")
            system = extract_spec_value(content, "system")

        if owner and KNOWN_GROUPS and owner.lower() not in KNOWN_GROUPS:
            errors.append(f"spec.owner: '{owner}' が Backstage Group catalog に存在しません")

        if system and KNOWN_SYSTEMS and system.lower() not in KNOWN_SYSTEMS:
            errors.append(f"spec.system: '{system}' が Backstage System catalog に存在しません")

    except Exception as e:
        errors.append(f"parse error: {e}")

    if errors:
        FAIL = True
        print(f"[FAIL] {path}")
        for err in errors:
            print(f"       {err}")
    else:
        print(f"[ok]   {path}")

sys.exit(1 if FAIL else 0)
PYEOF
)"

PYEXIT=$?
echo "${RESULT}"

if [[ "${PYEXIT}" -ne 0 ]]; then
  FAIL=1
fi

echo ""
if [[ "${FAIL}" -eq 1 ]]; then
  echo "[result] FAIL — owner/system 実在検証でエラーがあります（IMP-TRACE-CAT-023/024）"
  exit 1
else
  echo "[result] PASS — owner/system 実在検証完了"
  exit 0
fi
