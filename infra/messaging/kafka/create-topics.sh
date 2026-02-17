#!/bin/bash
# create-topics.sh
# ローカル開発環境 (docker-compose) 用の Kafka トピック作成スクリプト。
# docker-compose.yaml の kafka-init サービスから実行される。
#
# Kubernetes 環境では Strimzi KafkaTopic CRD (topics.yaml) を使用する。

set -euo pipefail

BOOTSTRAP_SERVER="${KAFKA_BOOTSTRAP_SERVER:-kafka:9092}"
REPLICATION_FACTOR="${KAFKA_REPLICATION_FACTOR:-1}"

echo "=== Creating Kafka topics (bootstrap: ${BOOTSTRAP_SERVER}) ==="

# --- System Tier ---
# 監査ログ (auth-server -> audit-aggregator)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.audit.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=7776000000

# 設定変更通知 (config-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.config.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 認証ログイン (auth-server)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.login.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- Service Tier ---
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.order.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.order.updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- DLQ Topics ---
for topic in \
  k1s0.system.auth.audit.v1.dlq \
  k1s0.system.config.changed.v1.dlq \
  k1s0.system.auth.login.v1.dlq \
  k1s0.service.order.created.v1.dlq \
  k1s0.service.order.updated.v1.dlq; do
  kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
    --create --if-not-exists \
    --topic "${topic}" \
    --partitions 1 \
    --replication-factor "${REPLICATION_FACTOR}" \
    --config retention.ms=2592000000
done

echo "=== All Kafka topics created successfully ==="

# トピック一覧を表示
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" --list
