#!/bin/bash
# create-topics.sh
# ローカル開発環境 (docker-compose) 用の Kafka トピック作成スクリプト。
# docker-compose.yaml の kafka-init サービスから実行される。
#
# Kubernetes 環境では Strimzi KafkaTopic CRD (topics.yaml) を使用する。
#
# M-005 監査対応: このスクリプトで定義するトピックは infra/messaging/kafka/topics.yaml と整合させること
# topics.yaml (Strimzi CRD) との差分チェックを定期的に実施し、両ファイルの同期を維持すること
# チェックコマンド例: diff <(grep "name:" topics.yaml | sort) <(grep "^kafka-topics" create-topics.sh | sort)
#
# M-03 監査対応: トピック命名の「.」と「_」の混在について確認済み（問題なし）
#
# トピック命名規則: {org}.{tier}.{service}.{event_name}.{version}
#   - 階層区切り: 「.」（ドット）を使用
#   - イベント名内の単語区切り: 「_」（アンダースコア）を使用
#   例: k1s0.system.auth.permission_denied.v1
#         ^^^^  ^^^^^^ ^^^^  ^^^^^^^^^^^^^^^^  ^^
#         org   tier  svc   event_name(複合語)  ver
#
# 「.」と「_」の混在は設計上の意図であり、以下の理由で問題なし:
#   1. Kafka はトピック名に「.」と「_」の両方を許可している
#   2. Kafka は「.」と「_」を区別し、混在による名前衝突は発生しない
#      （例: "foo.bar" と "foo_bar" は別トピックとして扱われる）
#   3. 「.」のみのトピックと「._」混在トピックが共存しているのは、
#      単一単語のイベント（changed, login 等）と複合単語のイベント（permission_denied 等）の違いによる
#   4. 将来的に同一名の「.」版と「_」版（例: foo.bar と foo_bar）を作成しない限り、
#      Kafka の WARN（KafkaException: Topic 'foo.bar' collides with 'foo_bar'）は発生しない
#
# パーティション数の設計方針:
# - 6 partitions (system tier 高優先度): 高スループットが必要なシステムイベント
#   (audit, config変更, auth, saga等) を対象とし、コンシューマーグループ最大6並列処理を想定。
# - 3 partitions (system tier 低優先度 / service tier): ファイル操作・クォータ等の
#   中程度トラフィックのシステムイベント、および業務イベント (task作成・更新等) を対象とし、
#   コンシューマーグループ最大3並列処理を想定。
# - 1 partition (DLQ): Dead Letter Queue は再処理時のメッセージ順序保証を優先し、
#   1並列処理で運用する。保持期間は30日 (retention.ms=2592000000)。
#
# B-MEDIUM-03 監査対応: 各トピックを & でバックグラウンド実行し並列化することで
#   JVM 起動オーバーヘッドを削減する（43回の順次実行 → 全並列実行後 wait で完了確認）。
#   wait コマンドの終了コードを明示的に確認し、いずれかのジョブ失敗時はスクリプトを失敗終了させる。

set -euo pipefail

BOOTSTRAP_SERVER="${KAFKA_BOOTSTRAP_SERVER:-kafka:9092}"
REPLICATION_FACTOR="${KAFKA_REPLICATION_FACTOR:-1}"

# HIGH-004 監査対応: 各バックグラウンドジョブの PID を追跡する配列
# wait だけでは最後に終了したジョブの終了コードしか返さないため、
# 各 PID を個別に wait で確認することで全ジョブの失敗を検出する。
declare -a PIDS=()

echo "=== Creating Kafka topics (bootstrap: ${BOOTSTRAP_SERVER}) ==="

# --- System Tier ---
# 監査ログ (auth-server -> audit-aggregator)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.audit.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=7776000000 &
PIDS+=($!)

# 設定変更通知 (config-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.config.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# 認証ログイン (auth-server)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.login.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# 権限拒否 (auth-server -> audit)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.permission_denied.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# APIレジストリ スキーマ更新 (api-registry -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.apiregistry.schema_updated.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# フィーチャーフラグ変更 (featureflag-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.featureflag.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# ファイルアップロード (file-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.file.uploaded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# ファイル削除 (file-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.file.deleted.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# ファイル汎用イベント（file-rust がプロデューサー、event_type ヘッダーで種別を区別）（M-21 監査対応）
# file-rust は topic_events 設定でこのトピックを使用する（uploaded.v1/deleted.v1 は将来のイベント分離用に残す）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.file.events.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# tenant イベントトピック（C-07 監査対応: データ損失防止のため追加）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.tenant.events.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# シークレットローテーション (vault-server -> subscribers)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.vault.secret_rotated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# 通知リクエスト (notification-server -> delivery)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.notification.requested.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# クォータ超過 (quota-server -> alerting)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.quota.exceeded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# Saga 状態変更 (saga-server -> orchestration)
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.saga.state_changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# トークン検証 (auth-server -> subscribers) ※topics.yaml k1s0.system.auth.token_validate.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.auth.token_validate.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# マスタデータ変更 (mastermaintenance-server -> subscribers) ※topics.yaml k1s0.system.mastermaintenance.data_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.mastermaintenance.data_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# --- Service Tier ---
# L-07 対応: topics.yaml との突合により task.updated.v1 / task.cancelled.v1 を追加する
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# タスク更新イベント ※topics.yaml k1s0.service.task.updated.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# タスクキャンセルイベント ※topics.yaml k1s0.service.task.cancelled.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.task.cancelled.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# board サービスのトピック
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.board.column_updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# activity サービスのトピック
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.activity.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.service.activity.approved.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# --- Business Tier ---
# L-07 対応: topics.yaml との突合により business tier トピックを追加する
# プロジェクト種別変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# ステータス定義変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 と対応
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# 検索インデックス更新イベント（search-rust がコンシューマー）（MED-1 監査対応）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.search.index.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# ワークフロー状態変更イベント（workflow-rust がプロデューサー）（MED-1 監査対応）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.workflow.state.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# スケジューライベント（scheduler-rust がプロデューサー）（M-20 監査対応: メイントピックが欠落していた）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# スケジューラ実行イベント（M-20 監査対応: メイントピックが欠落していた）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.executed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# スケジューラトリガーイベント（M-20 監査対応: メイントピックが欠落していた）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.triggered.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000 &
PIDS+=($!)

# スケジューラ作成 DLQ（MED-1 監査対応: scheduler.created.v1 に対する DLQ が欠落していた）
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.created.v1.dlq \
  --partitions 1 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=2592000000 &
PIDS+=($!)

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
  k1s0.system.file.events.v1.dlq \
  k1s0.system.tenant.events.v1.dlq \
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
  k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1.dlq \
  k1s0.system.search.index.v1.dlq \
  k1s0.system.workflow.state.v1.dlq \
  k1s0.system.scheduler.executed.v1.dlq \
  k1s0.system.scheduler.triggered.v1.dlq; do
  kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" \
    --create --if-not-exists \
    --topic "${topic}" \
    --partitions 1 \
    --replication-factor "${REPLICATION_FACTOR}" \
    --config retention.ms=2592000000 &
  PIDS+=($!)
done

# HIGH-004 監査対応: 各バックグラウンドジョブを PID 単位で個別確認する。
# wait だけでは最後に終了したジョブの終了コードしか返さず、途中のジョブ失敗を見逃す偽陽性が発生する。
# PID 配列をループして wait $pid で全ジョブの成否を確認し、一件でも失敗があればエラー終了する。
FAILED=0
for pid in "${PIDS[@]}"; do
  if ! wait "$pid"; then
    echo "ERROR: PID $pid のトピック作成に失敗しました" >&2
    FAILED=1
  fi
done
[ "$FAILED" -eq 0 ] || { echo "ERROR: 一部のトピック作成に失敗しました" >&2; exit 1; }

echo "=== All Kafka topics created successfully ==="

# トピック一覧を表示
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" --list
