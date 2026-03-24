#!/usr/bin/env bash
# M-06監査対応: コードベース内の TODO/FIXME にGitHub Issue番号が付与されているか確認する。
# Issue番号なしの TODO/FIXME は技術負債として追跡不能になるため、警告を出力する。
# CI では continue-on-error: true で実行（即時ブロッキングはしない）。
set -euo pipefail

echo "=== TODO/FIXME Issue 追跡チェック ==="

# Issue番号なしのTODO/FIXMEを検索（例: TODO: xxx は NG、TODO(#123): xxx は OK）
UNTRACKED=$(grep -rn --include="*.rs" --include="*.go" --include="*.ts" --include="*.dart" \
  -E "TODO[^(]|FIXME[^(]" regions/ CLI/ 2>/dev/null | \
  grep -v "TODO(#\|TODO(\!" | \
  wc -l)

echo "Issue 番号なし TODO/FIXME: ${UNTRACKED} 件"

if [ "$UNTRACKED" -gt 0 ]; then
  echo ""
  echo "上位10件のサンプル:"
  grep -rn --include="*.rs" --include="*.go" --include="*.ts" --include="*.dart" \
    -E "TODO[^(]|FIXME[^(]" regions/ CLI/ 2>/dev/null | \
    grep -v "TODO(#\|TODO(\!" | \
    head -10
  echo ""
  echo "[WARN] 技術負債として追跡するために GitHub Issue 番号を付与することを推奨します。"
  echo "  例: // TODO(#123): 実装が必要"
fi

echo "=== チェック完了 ==="
