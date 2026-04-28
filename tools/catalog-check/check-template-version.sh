#!/usr/bin/env bash
#
# tools/catalog-check/check-template-version.sh — k1s0.io/template-version annotation 検証
#
# 設計: docs/05_実装/99_索引/60_catalog-info検証/01_catalog-info検証設計.md
# 関連 IMP-ID: IMP-TRACE-CAT-021 / IMP-TRACE-CAT-027
# 責務:
#   全 catalog-info.yaml に metadata.annotations.k1s0.io/template-version が
#   存在し、SemVer 形式（v1.2.3）であることを検証する。
#   例外として "legacy-pre-scaffold" の固定値を許容する（設計書 CAT-021 明記）。
#   アノテーション欠落は即時 FAIL。
#
# Usage:
#   tools/catalog-check/check-template-version.sh [--strict] [--help]
#   tools/catalog-check/check-template-version.sh [FILE ...]
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -h|--help)
      sed -n '3,22p' "$0" | sed 's/^# \{0,1\}//'
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

RESULT="$(python3 - "${TARGET_FILES[@]}" <<'PYEOF'
import sys, re

SEMVER_RE = re.compile(r'^v\d+\.\d+\.\d+$')
LEGACY_VALUE = "legacy-pre-scaffold"
ANNOTATION_KEY = "k1s0.io/template-version"
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
        else:
            doc = {}
            # 簡易: annotation キーを直接 grep
            for line in content.split('\n'):
                if ANNOTATION_KEY in line:
                    val = line.split(':', 1)[-1].strip().strip('"\'')
                    doc = {"_annotation_val": val}
                    break

        if doc is None:
            doc = {}

        if has_yaml:
            metadata = doc.get("metadata") or {}
            annotations = metadata.get("annotations") or {}
            val = annotations.get(ANNOTATION_KEY, "")
        else:
            val = doc.get("_annotation_val", "")

        if not val:
            errors.append(f"{ANNOTATION_KEY}: 未設定（Scaffold 生成後の手動削除または未設定）")
        elif val != LEGACY_VALUE and not SEMVER_RE.match(val):
            errors.append(f"{ANNOTATION_KEY}: '{val}' は SemVer (vX.Y.Z) でも 'legacy-pre-scaffold' でもありません")

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
  echo "[result] FAIL — k1s0.io/template-version annotation 違反（IMP-TRACE-CAT-021）"
  echo "         修正候補: Scaffold CLI で再生成するか 'legacy-pre-scaffold' を設定"
  exit 1
else
  echo "[result] PASS — template-version annotation 検証完了"
  exit 0
fi
