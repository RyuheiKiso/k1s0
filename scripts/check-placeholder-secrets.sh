#!/usr/bin/env bash
# CI/CD パイプラインでプレースホルダー値の存在を検出し、デプロイを阻止するスクリプト（CRITICAL-SEC-01 監査対応）
# encryption-config.yaml に <REPLACE_ パターンが残存している場合はエラーで終了する
set -euo pipefail

TARGET_FILE="${1:-infra/kubernetes/security/encryption-config.yaml}"

if [ ! -f "$TARGET_FILE" ]; then
  echo "INFO: $TARGET_FILE が見つかりません。テンプレートファイルのみが存在する場合は問題ありません。"
  exit 0
fi

if grep -qn '<REPLACE_' "$TARGET_FILE"; then
  echo "ERROR: $TARGET_FILE にプレースホルダー値が残存しています。本番デプロイ前に Vault から実キーを注入してください。"
  grep -n '<REPLACE_' "$TARGET_FILE"
  exit 1
fi

echo "OK: $TARGET_FILE にプレースホルダー値は検出されませんでした。"
exit 0
