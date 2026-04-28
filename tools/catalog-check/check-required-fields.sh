#!/usr/bin/env bash
#
# tools/catalog-check/check-required-fields.sh — catalog-info.yaml 必須属性スキーマ検証
#
# 設計: docs/05_実装/99_索引/60_catalog-info検証/01_catalog-info検証設計.md
# 関連 IMP-ID: IMP-TRACE-CAT-020 / IMP-TRACE-CAT-027
# 責務:
#   リポジトリ内の全 catalog-info.yaml について以下を検証する:
#   - apiVersion: backstage.io/v1alpha1 の固定値
#   - kind: Component | API | Group | System の許可リスト
#   - metadata.name の存在と kebab-case 正規表現（^[a-z0-9-]+$）
#   - metadata.namespace が default のみ
#   - spec.owner の存在
#   - spec.lifecycle の存在（値は check-lifecycle.sh で詳細検証）
#   スキーマ違反は即時 FAIL とし、欠落属性を列挙する。
#
# Usage:
#   tools/catalog-check/check-required-fields.sh [--strict] [--help]
#   tools/catalog-check/check-required-fields.sh [FILE ...]
#
# Exit code:
#   0 = pass
#   1 = fail（スキーマ違反あり）
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

# ──────────────────────────────────────────────────────────
# 依存チェック
# ──────────────────────────────────────────────────────────
if ! command -v python3 >/dev/null 2>&1; then
  echo "[setup-error] python3 が見つかりません" >&2
  exit 2
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
echo "[info] 検証対象 catalog-info.yaml: ${total} 件"

if [[ "${total}" -eq 0 ]]; then
  echo "[warn] catalog-info.yaml が 1 件も見つかりません"
  exit 0
fi

# ──────────────────────────────────────────────────────────
# Python による YAML 必須属性チェック
# ──────────────────────────────────────────────────────────
RESULT="$(python3 - "${TARGET_FILES[@]}" <<'PYEOF'
import sys, re

ALLOWED_KIND       = {"Component", "API", "Group", "System"}
ALLOWED_APIVERSION = "backstage.io/v1alpha1"
NAME_RE            = re.compile(r'^[a-z0-9][a-z0-9-]*$')
FAIL               = False

try:
    import yaml
    has_yaml = True
except ImportError:
    has_yaml = False

def parse_yaml_simple(content):
    """PyYAML なし時の最小限 YAML パーサ（catalog-info.yaml 程度なら十分）"""
    result = {}
    stack = [result]
    indent_map = {0: result}
    current_indent = 0
    lines = content.split('\n')
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.lstrip()
        if not stripped or stripped.startswith('#'):
            i += 1
            continue
        indent = len(line) - len(stripped)
        if ':' in stripped:
            key, _, val = stripped.partition(':')
            key = key.strip()
            val = val.strip().strip('"\'')
            # ネストは記録しない（簡易版はトップレベルと1段のみ追う）
            if indent == 0:
                if val:
                    result[key] = val
                else:
                    result[key] = {}
            elif indent in indent_map and isinstance(indent_map.get(indent-2, result), dict):
                parent = indent_map.get(indent - 2, result)
                if val:
                    parent[key] = val
                else:
                    parent[key] = {}
                    indent_map[indent] = parent[key]
        i += 1
    return result

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
            doc = parse_yaml_simple(content)
        if doc is None:
            doc = {}

        # apiVersion
        av = doc.get("apiVersion", "")
        if av != ALLOWED_APIVERSION:
            errors.append(f"apiVersion: '{av}' ≠ '{ALLOWED_APIVERSION}'")

        # kind
        kind = doc.get("kind", "")
        if kind not in ALLOWED_KIND:
            errors.append(f"kind: '{kind}' は許可リスト外 (許可: {sorted(ALLOWED_KIND)})")

        # metadata
        metadata = doc.get("metadata") or {}
        if not isinstance(metadata, dict):
            errors.append("metadata: 辞書型でない")
            metadata = {}

        name = metadata.get("name", "")
        if not name:
            errors.append("metadata.name: 未設定")
        elif not NAME_RE.match(name):
            errors.append(f"metadata.name: '{name}' は kebab-case (^[a-z0-9][a-z0-9-]*$) に違反")

        ns = metadata.get("namespace", "default")
        if ns and ns != "default":
            errors.append(f"metadata.namespace: '{ns}' ≠ 'default'")

        # spec
        spec = doc.get("spec") or {}
        if not isinstance(spec, dict):
            errors.append("spec: 辞書型でない")
            spec = {}

        if not spec.get("owner"):
            errors.append("spec.owner: 未設定")

        if not spec.get("lifecycle"):
            errors.append("spec.lifecycle: 未設定")

    except Exception as e:
        errors.append(f"parse error: {e}")

    rel_path = path
    if errors:
        FAIL = True
        print(f"[FAIL] {rel_path}")
        for err in errors:
            print(f"       {err}")
    else:
        print(f"[ok]   {rel_path}")

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
  echo "[result] FAIL — 必須属性スキーマ違反があります（IMP-TRACE-CAT-020）"
  exit 1
else
  echo "[result] PASS — 全 catalog-info.yaml の必須属性検証完了"
  exit 0
fi
