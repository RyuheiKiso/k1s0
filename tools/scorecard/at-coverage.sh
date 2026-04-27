#!/usr/bin/env bash
# 本ファイルは AnalysisTemplate カバレッジ計測スクリプト（IMP-REL-AT-048）。
# 月次バッチで全サービスの AnalysisTemplate / Rollout を走査し、
# 共通 5 本（at-common-*）と固有テンプレの参照状況から SLI カバレッジを算出する。
# カバレッジ 70% 未満のサービスは Backstage TechInsights Scorecard に warning 表示し、
# `docs/04_概要設計/60_観測性設計/` の SLO 設計レビュー対象として候補出しする。
#
# 出力: stdout に JSON Lines 形式で 1 サービス 1 レコード、Backstage TechInsights が parse する。
#   {"service": "tier1-state", "namespace": "k1s0-tier1", "common_count": 5, "specific_count": 1,
#    "coverage": 0.83, "verdict": "ok"}
#
# 詳細設計: docs/05_実装/70_リリース設計/40_AnalysisTemplate/01_AnalysisTemplate設計.md
#
# usage:
#   tools/scorecard/at-coverage.sh                    # 全 namespace 走査
#   tools/scorecard/at-coverage.sh --namespace k1s0-tier1
#   tools/scorecard/at-coverage.sh --output /tmp/at-coverage.jsonl
#   tools/scorecard/at-coverage.sh --threshold 0.7    # warn しきい値変更
set -euo pipefail

# 既定値。
NAMESPACE_FILTER=""
OUTPUT_PATH="-"
THRESHOLD="0.7"
COMMON_TEMPLATES=(
  "at-common-error-rate"
  "at-common-latency-p99"
  "at-common-cpu"
  "at-common-dependency-down"
  "at-common-error-budget-burn"
)

# CLI フラグの解析。
while [ "$#" -gt 0 ]; do
  case "$1" in
    --namespace)
      # 単一 namespace を対象にする。
      NAMESPACE_FILTER="$2"
      shift 2
      ;;
    --output)
      # 出力先（- は stdout）。
      OUTPUT_PATH="$2"
      shift 2
      ;;
    --threshold)
      # warning しきい値（小数）。
      THRESHOLD="$2"
      shift 2
      ;;
    --help|-h)
      # ヘルプ表示。
      grep '^# ' "$0" | sed 's/^# //'
      exit 0
      ;;
    *)
      # 未知フラグはエラー。
      echo "unknown flag: $1" >&2
      exit 2
      ;;
  esac
done

# 必須コマンドの存在確認。
require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command missing: $1" >&2; exit 3; }
}
require_cmd kubectl
require_cmd jq

# 出力先のセットアップ。
if [ "$OUTPUT_PATH" != "-" ]; then
  exec >"$OUTPUT_PATH"
fi

# Rollout を持つ namespace を列挙。
list_namespaces() {
  if [ -n "$NAMESPACE_FILTER" ]; then
    echo "$NAMESPACE_FILTER"
  else
    kubectl get rollouts -A -o jsonpath='{range .items[*]}{.metadata.namespace}{"\n"}{end}' | sort -u
  fi
}

# 単一 Rollout のカバレッジを評価する。
emit_one() {
  local ns="$1"
  local name="$2"
  # Rollout の analysis.templates 配列を取得。
  local raw
  raw="$(kubectl get rollout -n "$ns" "$name" -o json 2>/dev/null || true)"
  if [ -z "$raw" ]; then
    return
  fi

  # 参照されている template 名を抽出。
  local refs
  refs="$(printf '%s' "$raw" | jq -r '
      [
        (.spec.strategy.canary.steps // [])[] | select(.analysis != null) |
        .analysis.templates[] | .templateName
      ] | unique | .[]
    ' 2>/dev/null || true)"

  # 共通テンプレ参照数をカウント。
  local common=0
  for t in "${COMMON_TEMPLATES[@]}"; do
    if printf '%s\n' "$refs" | grep -qx "$t"; then
      common=$((common + 1))
    fi
  done

  # 固有テンプレ数（共通でない参照）。
  local specific=0
  while IFS= read -r r; do
    [ -z "$r" ] && continue
    if ! printf '%s ' "${COMMON_TEMPLATES[@]}" | grep -qw "$r"; then
      specific=$((specific + 1))
    fi
  done <<< "$refs"

  # カバレッジは「共通 5 本のうちいくつ参照しているか」を 5 で割って 0〜1 に正規化。
  # 固有テンプレは加点しない（共通の SLI 軸 5 つを満たすことを最重視する設計）。
  local coverage
  coverage="$(awk -v c="$common" 'BEGIN { printf "%.4f", c/5 }')"

  # しきい値判定。
  local verdict="ok"
  if awk -v c="$coverage" -v t="$THRESHOLD" 'BEGIN { exit !(c<t) }'; then
    verdict="warn"
  fi

  # JSON Lines を出力。
  jq -nc --arg ns "$ns" --arg name "$name" \
        --argjson common "$common" --argjson specific "$specific" \
        --arg coverage "$coverage" --arg verdict "$verdict" \
        '{namespace:$ns, service:$name, common_count:$common, specific_count:$specific,
          coverage:($coverage|tonumber), verdict:$verdict}'
}

# メイン: 各 namespace の Rollout を順次評価。
list_namespaces | while IFS= read -r ns; do
  [ -z "$ns" ] && continue
  kubectl get rollouts -n "$ns" -o jsonpath='{range .items[*]}{.metadata.name}{"\n"}{end}' \
    | while IFS= read -r name; do
        [ -z "$name" ] && continue
        emit_one "$ns" "$name"
      done
done
