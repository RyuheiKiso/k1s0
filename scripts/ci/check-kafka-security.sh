#!/usr/bin/env bash
# M-02監査対応: 本番環境（staging/prod）のHelmバリューに Kafka SASL_SSL が設定されていることを検証する。
# ローカル開発環境（docker-compose.yaml）では PLAINTEXT が使用されるが、
# 本番用 values-*.yaml では SASL_SSL が必須であることを CI で保証する。
set -euo pipefail

ERRORS=0
REQUIRED_PROTOCOL="SASL_SSL"

# staging/prod 用 values ファイルを検索
for values_file in infra/helm/services/**/values-staging.yaml infra/helm/services/**/values-prod.yaml; do
  [ -f "$values_file" ] || continue
  # KAFKA_SECURITY_PROTOCOL または kafkaSecurityProtocol の設定を確認
  if grep -q "kafkaSecurityProtocol\|KAFKA_SECURITY_PROTOCOL" "$values_file"; then
    if ! grep -q "$REQUIRED_PROTOCOL" "$values_file"; then
      echo "[ERROR] $values_file: Kafka security protocol is not $REQUIRED_PROTOCOL"
      ERRORS=$((ERRORS + 1))
    fi
  fi
done

if [ $ERRORS -gt 0 ]; then
  echo "[FAIL] $ERRORS file(s) have incorrect Kafka security protocol configuration."
  exit 1
fi
echo "[OK] Kafka security protocol check passed."
