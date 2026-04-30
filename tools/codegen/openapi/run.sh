#!/usr/bin/env bash
#
# tools/codegen/openapi/run.sh — proto → OpenAPI v2 (Swagger) export
#
# 設計: plan/03_Contracts実装/06_OpenAPI_export.md
# 関連 ID: IMP-CODEGEN-006（OpenAPI export）/ ADR-BS-001（Backstage 連携）
#
# 出力:
#   docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml
#   （allow_merge=true で 12 API + health + 共通型を 1 yaml にマージ）
#
# Usage:
#   tools/codegen/openapi/run.sh         # 通常生成
#   tools/codegen/openapi/run.sh --check # diff 検出のみ（CI 用）

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${REPO_ROOT}"

CHECK=0
for arg in "$@"; do
    case "$arg" in
        --check) CHECK=1 ;;
        -h|--help)
            sed -n '3,15p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "[error] 未知の引数: $arg" >&2
            exit 1
            ;;
    esac
done

if ! command -v buf >/dev/null 2>&1; then
    echo "[error] buf CLI が見つかりません" >&2
    exit 1
fi

# 出力ディレクトリ準備
mkdir -p docs/02_構想設計/02_tier1設計/openapi/v1

# BSR remote plugin の rate limit に対する retry。詳細は tools/codegen/buf/run.sh
# の buf_generate_with_retry と同じ方針（30s, 60s, 90s で 3 回まで）。
buf_generate_with_retry() {
    local label="$1"; shift
    local attempt=1
    local max_attempt=3
    while :; do
        local out
        if out="$(buf generate "$@" 2>&1)"; then
            [[ -n "$out" ]] && echo "$out"
            return 0
        fi
        if [[ "$attempt" -lt "$max_attempt" ]] && grep -qiE 'too many requests|rate limit|429' <<< "$out"; then
            local sleep_sec=$((attempt * 30))
            echo "[warn] ${label}: BSR rate limit 検出。${sleep_sec}s 後に retry (${attempt}/${max_attempt})" >&2
            sleep "${sleep_sec}"
            attempt=$((attempt + 1))
            continue
        fi
        echo "$out" >&2
        return 1
    done
}

echo "[info] buf generate (OpenAPI v2)"
buf_generate_with_retry "openapi" --template buf.gen.openapi.yaml

# 出力ファイルの確認
out_file="docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml"
if [[ -f "${out_file}" ]]; then
    lines=$(wc -l < "${out_file}")
    echo "[ok] 出力: ${out_file} (${lines} 行)"
else
    echo "[warn] 期待出力 ${out_file} が見つかりません"
    find docs/02_構想設計/02_tier1設計/openapi/v1 -name "*.yaml" -o -name "*.yml" -o -name "*.json" 2>/dev/null
fi

# tier1 全 RPC handler は TenantContext.tenant_id を「非空文字列」で要求する
# （docs §「マルチテナント分離」/ tier1 facade implementation）。proto 側では
# google.api.field_behavior = REQUIRED で「キー存在必須」を表現したが、OpenAPI v2
# には「非空」を表現する正準な field_behavior 拡張が無いため、生成 yaml に対して
# minLength: 1 を post-inject する。schemathesis 等が「空文字列 tenant_id を impl が
# 拒否する」を contract drift として誤検出しないよう、spec 側で非空制約を明示する。
echo "[info] post-process: v1TenantContext.tenant_id に minLength: 1 を注入"
python3 - "${out_file}" <<'PY'
# tools/codegen/openapi/run.sh から呼ばれる post-processor。
import sys, re, pathlib
p = pathlib.Path(sys.argv[1])
s = p.read_text(encoding="utf-8")
# v1TenantContext ブロック内の tenant_id プロパティに minLength: 1 を追加する。
# yaml 構造:
#   v1TenantContext:
#     type: object
#     properties:
#       tenant_id:
#         type: string
#         title: |- ... 複数行
#       subject: ...
# title が複数行（|-）なため、tenant_id ブロックの開始から次の同レベル property（subject）
# 直前までを検出して、type: string の直後に minLength: 1 を挿入する。
# 重複注入を避けるため、既に minLength を持つ場合はスキップする。
needle = "  v1TenantContext:\n"
i = s.find(needle)
if i < 0:
    print("[error] v1TenantContext が見つかりません", file=sys.stderr)
    sys.exit(1)
# 当該ブロックの tenant_id を探す
block_start = i
# 次の top-level definition（インデント 2 スペース + 識別子 + ':'）の手前まで
m = re.search(r"\n  [A-Za-z][A-Za-z0-9]*:\n", s[block_start + len(needle):])
block_end = (block_start + len(needle) + m.start()) if m else len(s)
block = s[block_start:block_end]
if "minLength: 1" not in block:
    # tenant_id プロパティの "type: string" 行（インデント 8 スペース）を取得
    new_block = re.sub(
        r"(      tenant_id:\n        type: string\n)",
        r"\1        minLength: 1\n",
        block,
        count=1,
    )
    if new_block == block:
        print("[error] tenant_id への min_length 注入に失敗", file=sys.stderr)
        sys.exit(1)
    s = s[:block_start] + new_block + s[block_end:]
    print("[ok] tenant_id に minLength: 1 を注入")
else:
    print("[ok] minLength: 1 は既に注入済み")

# 第 2 注入: 各 *Request message が `context: $ref v1TenantContext` を含む場合、
# その Request schema の最後に `required: [context]` を追加する。tier1 全 RPC handler は
# context.tenant_id を必須として弾く（NFR-E-AC-003 / requireTenantID）。schemathesis は
# `required` が明示されない限り「context 抜きで送信」を schema-compliant と判定し、
# impl の InvalidArgument を contract drift として誤検出するため、spec 側で context 必須を
# 明示する。protoc-gen-openapiv2 は field_behavior REQUIRED を field 単位でしか
# 反映しないため、ここで post-process する。
import yaml as _yaml  # PyYAML は標準的に CI 環境にある（schemathesis も依存）
data = _yaml.safe_load(s)
defs = data.get("definitions", {})
modified = 0
for name, schema in list(defs.items()):
    if not isinstance(schema, dict):
        continue
    props = schema.get("properties") or {}
    ctx = props.get("context")
    if not isinstance(ctx, dict):
        continue
    if ctx.get("$ref") != "#/definitions/v1TenantContext":
        continue
    req = schema.get("required") or []
    if "context" in req:
        continue
    req.append("context")
    schema["required"] = sorted(req)
    modified += 1
if modified > 0:
    s = _yaml.safe_dump(data, allow_unicode=True, sort_keys=False, width=10000)
    p.write_text(s, encoding="utf-8")
    print(f"[ok] {modified} 件の *Request schema に required: [context] を注入")
else:
    p.write_text(s, encoding="utf-8")
    print("[ok] context 必須注入対象なし（既に追記済み）")
PY

# tests/contract/openapi-contract/tier1-openapi-spec.yaml は spec の物理コピー。
# schemathesis / dredd 等の契約検証ツールが対象として参照する。
# README の規約（「上記の物理コピー」）と一致させるため、生成と同時に同期する。
contract_copy='tests/contract/openapi-contract/tier1-openapi-spec.yaml'
if [[ -f "${out_file}" ]]; then
    cp "${out_file}" "${contract_copy}"
    echo "[ok] 同期: ${contract_copy}"
fi

if [[ "${CHECK}" == "1" ]]; then
    # 1) docs / contract コピーの両方の drift を検出
    target='docs/02_構想設計/02_tier1設計/openapi tests/contract/openapi-contract'
    # 1) 既追跡ファイルの変更検出
    if ! git diff --exit-code -- ${target}; then
        echo "[error] OpenAPI が最新でありません。"
        echo "  対処: tools/codegen/openapi/run.sh を再実行し、git add してください。"
        exit 1
    fi
    # 2) untracked（新規生成）も検出。proto 追加で OpenAPI が増えた時の取りこぼし防止。
    untracked=$(git ls-files --others --exclude-standard -- ${target})
    if [[ -n "${untracked}" ]]; then
        echo "[error] OpenAPI に未追跡ファイルがあります。git add してください:" >&2
        echo "${untracked}" >&2
        exit 1
    fi
    echo "[ok] OpenAPI diff なし"
fi
