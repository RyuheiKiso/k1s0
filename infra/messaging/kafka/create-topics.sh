#!/bin/bash
# create-topics.sh
# ローカル開発環境 (docker-compose) 用の Kafka トピック作成スクリプト。
# docker-compose.yaml の kafka-init サービスから実行される。
#
# Kubernetes 環境では Strimzi KafkaTopic CRD (topics.yaml) を使用する。
#
# パーティション数の設計方針:
# - 6 partitions (system tier 高優先度): 高スループットが必要なシステムイベント
#   (audit, config変更, auth, saga等) を対象とし、コンシューマーグループ最大6並列処理を想定。
# - 3 partitions (system tier 低優先度 / service tier): ファイル操作・クォータ等の
#   中程度トラフィックのシステムイベント、および業務イベント (task作成・更新等) を対象とし、
#   コンシューマーグループ最大3並列処理を想定。
# - 1 partition (DLQ): Dead Letter Queue は再処理時のメッセージ順序保証を優先し、
#   1並列処理で運用する。保持期間は30日 (retention.ms=2592000000)。

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

# 権限拒否 (auth-server -> audit)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.permission_denied.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# APIレジストリ スキーマ更新 (api-registry -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.apiregistry.schema_updated.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# フィーチャーフラグ変更 (featureflag-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.featureflag.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ファイルアップロード (file-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.file.uploaded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ファイル削除 (file-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.file.deleted.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# シークレットローテーション (vault-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.vault.secret_rotated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 通知リクエスト (notification-server -> delivery)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.notification.requested.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# クォータ超過 (quota-server -> alerting)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.quota.exceeded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# Saga 状態変更 (saga-server -> orchestration)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.saga.state_changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# トークン検証 (auth-server -> subscribers) ※topics.yaml k1s0.system.auth.token_validate.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.token_validate.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# マスタデータ変更 (mastermaintenance-server -> subscribers) ※topics.yaml k1s0.system.mastermaintenance.data_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.mastermaintenance.data_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- Service Tier ---
# L-07 対応: topics.yaml との突合により task.updated.v1 / task.cancelled.v1 を追加する
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# タスク更新イベント ※topics.yaml k1s0.service.task.updated.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# タスクキャンセルイベント ※topics.yaml k1s0.service.task.cancelled.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.cancelled.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# board サービスのトピック
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.board.column_updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# activity サービスのトピック
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.activity.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.activity.approved.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- Business Tier ---
# L-07 対応: topics.yaml との突合により business tier トピックを追加する
# プロジェクト種別変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ステータス定義変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- DLQ Topics ---
# L-07 対応: topics.yaml との突合により不足 DLQ を追加する
for topic in \
  k1s0.system.auth.audit.v1.dlq \
  k1s0.system.config.changed.v1.dlq \
  k1s0.system.auth.login.v1.dlq \
  k1s0.system.auth.token_validate.v1.dlq \
  k1s0.system.auth.permission_denied.v1.dlq \
  k1s0.system.apiregistry.schema_updated.v1.dlq \
  k1s0.system.mastermaintenance.data_changed.v1.dlq \
  k1s0.system.featureflag.changed.v1.dlq \
  k1s0.system.file.uploaded.v1.dlq \
  k1s0.system.file.deleted.v1.dlq \
  k1s0.system.vault.secret_rotated.v1.dlq \
  k1s0.system.notification.requested.v1.dlq \
  k1s0.system.quota.exceeded.v1.dlq \
  k1s0.system.saga.state_changed.v1.dlq \
  k1s0.service.task.created.v1.dlq \
  k1s0.service.task.updated.v1.dlq \
  k1s0.service.task.cancelled.v1.dlq \
  k1s0.service.board.column_updated.v1.dlq \
  k1s0.service.activity.created.v1.dlq \
  k1s0.service.activity.approved.v1.dlq \
  k1s0.business.taskmanagement.projectmaster.project_type_changed.v1.dlq \
  k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1.dlq; do
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
