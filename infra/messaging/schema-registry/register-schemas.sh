#!/bin/bash
# register-schemas.sh
# Confluent Schema Registry に Protobuf スキーマを登録するスクリプト。
# ローカル開発環境 (docker-compose) で使用する。
#
# 前提: Schema Registry が起動済みであること。
#   docker compose --profile infra up -d
#
# 使い方:
#   bash infra/messaging/schema-registry/register-schemas.sh

set -euo pipefail

SCHEMA_REGISTRY_URL="${SCHEMA_REGISTRY_URL:-http://localhost:8081}"
PROTO_ROOT="api/proto"

echo "=== Schema Registry: ${SCHEMA_REGISTRY_URL} ==="
echo "=== Registering Protobuf schemas ==="

# 共通メタデータスキーマを参照として登録
EVENT_METADATA_PROTO=$(cat "${PROTO_ROOT}/k1s0/system/common/v1/event_metadata.proto")

# Subject 命名規則: {topic-name}-value
# 互換性モード: BACKWARD (デフォルト)

# --- k1s0.system.auth.audit.v1-value ---
# auth_events.proto は event_metadata.proto を参照するため、references を使用する。
echo "Registering event_metadata schema as reference..."
curl -s -X POST "${SCHEMA_REGISTRY_URL}/subjects/k1s0.system.common.event-metadata.v1/versions" \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d "$(jq -n \
    --arg schema "${EVENT_METADATA_PROTO}" \
    '{schemaType: "PROTOBUF", schema: $schema}'
  )"
echo ""

echo "Registering k1s0.system.auth.audit.v1-value..."
AUTH_EVENTS_PROTO=$(cat "${PROTO_ROOT}/k1s0/event/system/auth/v1/auth_events.proto")
curl -s -X POST "${SCHEMA_REGISTRY_URL}/subjects/k1s0.system.auth.audit.v1-value/versions" \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d "$(jq -n \
    --arg schema "${AUTH_EVENTS_PROTO}" \
    '{
      schemaType: "PROTOBUF",
      schema: $schema,
      references: [
        {
          name: "k1s0/system/common/v1/event_metadata.proto",
          subject: "k1s0.system.common.event-metadata.v1",
          version: 1
        }
      ]
    }'
  )"
echo ""

echo "Registering k1s0.system.auth.login.v1-value..."
curl -s -X POST "${SCHEMA_REGISTRY_URL}/subjects/k1s0.system.auth.login.v1-value/versions" \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d "$(jq -n \
    --arg schema "${AUTH_EVENTS_PROTO}" \
    '{
      schemaType: "PROTOBUF",
      schema: $schema,
      references: [
        {
          name: "k1s0/system/common/v1/event_metadata.proto",
          subject: "k1s0.system.common.event-metadata.v1",
          version: 1
        }
      ]
    }'
  )"
echo ""

# --- k1s0.system.config.changed.v1-value ---
echo "Registering k1s0.system.config.changed.v1-value..."
CONFIG_EVENTS_PROTO=$(cat "${PROTO_ROOT}/k1s0/event/system/config/v1/config_events.proto")
curl -s -X POST "${SCHEMA_REGISTRY_URL}/subjects/k1s0.system.config.changed.v1-value/versions" \
  -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d "$(jq -n \
    --arg schema "${CONFIG_EVENTS_PROTO}" \
    '{
      schemaType: "PROTOBUF",
      schema: $schema,
      references: [
        {
          name: "k1s0/system/common/v1/event_metadata.proto",
          subject: "k1s0.system.common.event-metadata.v1",
          version: 1
        }
      ]
    }'
  )"
echo ""

# 互換性モードを確認・設定
echo "Setting compatibility mode to BACKWARD for all subjects..."
for subject in \
  "k1s0.system.auth.audit.v1-value" \
  "k1s0.system.auth.login.v1-value" \
  "k1s0.system.config.changed.v1-value"; do
  curl -s -X PUT "${SCHEMA_REGISTRY_URL}/config/${subject}" \
    -H "Content-Type: application/vnd.schemaregistry.v1+json" \
    -d '{"compatibility": "BACKWARD"}'
  echo ""
done

echo ""
echo "=== Listing registered subjects ==="
curl -s "${SCHEMA_REGISTRY_URL}/subjects" | jq .

echo ""
echo "=== Schema registration complete ==="
