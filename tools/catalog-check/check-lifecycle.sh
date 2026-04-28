#!/usr/bin/env bash
#
# tools/catalog-check/check-lifecycle.sh — spec.lifecycle 許可リスト検証
#
# 設計: docs/05_実装/99_索引/60_catalog-info検証/01_catalog-info検証設計.md
# 関連 IMP-ID: IMP-TRACE-CAT-022 / IMP-TRACE-CAT-027
# 責務:
#   全 catalog-info.yaml の spec.lifecycle が以下の 3 値のいずれかであることを検証する:
#   - experimental: Scaffold 直後 / リリース時点 examples / 評価段階
#   - production:   tier1/tier2/tier3 本番稼働 component（DORA 計測対象）
#   - deprecated:   廃止予定 component
#   許可リスト外の値（alpha / beta / staging 等）は即時 FAIL。
#
# Usage:
#   tools/catalog-check/check-lifecycle.sh [--strict] [--help]
#   tools/catalog-check/check-lifecycle.sh [FILE ...]
#
# Exit code:
#   0 = pass
#   1 = fail
#   2 = setup error

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STRICT=0
FAIL=0
TARGET_FILES=()
ALLOWED_LIFECYCLES=("experimental" "production" "deprecated")

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -h|--help)
      sed -n '3,24p' "$0" | sed 's/^# \{0,1\}//'
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
echo "[info] 許可 lifecycle: ${ALLOWED_LIFECYCLES[*]}"

RESULT="$(python3 - "${TARGET_FILES[@]}" <<'PYEOF'
import sys

ALLOWED = {"experimental", "production", "deprecated"}
FAIL = False

try:
    import yaml
    has_yaml = True
except ImportError:
    has_yaml = False

files = sys.argv[1:]
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
            lifecycle = spec.get("lifecycle", "")
        else:
            # 簡易パース: "lifecycle: xxx" を探す
            lifecycle = ""
            for line in content.split('\n'):
                stripped = line.strip()
                if stripped.startswith("lifecycle:"):
                    lifecycle = stripped.split(":", 1)[-1].strip().strip('"\'')
                    break

        if not lifecycle:
            errors.append("spec.lifecycle: 未設定")
        elif lifecycle not in ALLOWED:
            errors.append(
                f"spec.lifecycle: '{lifecycle}' は許可リスト外 "
                f"(許可: {sorted(ALLOWED)})"
            )

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
  echo "[result] FAIL — spec.lifecycle 許可リスト違反（IMP-TRACE-CAT-022）"
  echo "         修正候補: experimental / production / deprecated のいずれかに変更"
  exit 1
else
  echo "[result] PASS — spec.lifecycle 許可リスト検証完了"
  exit 0
fi
