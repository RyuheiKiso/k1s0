#!/usr/bin/env bash
# CI/CD パイプラインでプレースホルダー値の存在を検出し、デプロイを阻止するスクリプト（CRITICAL-SEC-01 監査対応）
# 以下のファイルを検査対象とする:
#   - infra/kubernetes/security/encryption-config.yaml              : etcd 暗号化キー（CRIT-5 監査対応）
#   - infra/kubernetes/ingress/kong-consumer-grafana.yaml           : Grafana API キー（CRIT-6 監査対応）
#   - infra/observability/alertmanager/prometheus-msteams-webhook-secret.yaml : Webhook URL（C-04 監査対応）
# <REPLACE_ パターンが残存している場合はエラーで終了する
set -euo pipefail

# 検査対象ファイルの一覧（引数が与えられた場合はそのファイルのみ検査する）
if [ $# -gt 0 ]; then
  TARGET_FILES=("$@")
else
  # デフォルト: 全てのプレースホルダー検査対象ファイルを検査する
  TARGET_FILES=(
    "infra/kubernetes/security/encryption-config.yaml"
    "infra/kubernetes/ingress/kong-consumer-grafana.yaml"
    "infra/observability/alertmanager/prometheus-msteams-webhook-secret.yaml"
  )
fi

# 検査全体の成否を追跡するフラグ
FOUND_ERROR=0

for TARGET_FILE in "${TARGET_FILES[@]}"; do
  if [ ! -f "$TARGET_FILE" ]; then
    # ファイルが存在しない場合はスキップ（テンプレートのみの環境を考慮）
    echo "INFO: $TARGET_FILE が見つかりません。テンプレートファイルのみが存在する場合は問題ありません。"
    continue
  fi

  if grep -qn '<REPLACE_' "$TARGET_FILE"; then
    echo "ERROR: $TARGET_FILE にプレースホルダー値が残存しています。本番デプロイ前に Vault から実値を注入してください。"
    grep -n '<REPLACE_' "$TARGET_FILE"
    FOUND_ERROR=1
  else
    echo "OK: $TARGET_FILE にプレースホルダー値は検出されませんでした。"
  fi
done

# 1 件でもエラーがあれば異常終了してデプロイを阻止する
if [ "$FOUND_ERROR" -ne 0 ]; then
  exit 1
fi

exit 0
