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
# CRIT-001 監査対応: 並列 JVM 数を制限してコンテナ OOM を防止し、失敗時に exit 1 を返す
#   - MAX_PARALLEL=5 でセマフォ制御（57 JVM 同時起動は mem_limit=1.5g でOOM必至）
#   - ACTUAL_COUNT が期待値未満の場合は exit 1 で失敗終了し、
#     docker-compose の restart: "on-failure:3" でリトライさせる
#   - WARN_COUNT が閾値を超えた場合も exit 1 で失敗終了する

set -euo pipefail

BOOTSTRAP_SERVER="${KAFKA_BOOTSTRAP_SERVER:-kafka:9092}"
REPLICATION_FACTOR="${KAFKA_REPLICATION_FACTOR:-1}"
# CRIT-001 監査対応: 同時起動 JVM 数の上限（OOM 防止）
MAX_PARALLEL="${MAX_PARALLEL:-5}"
# WARN_COUNT の許容上限（--if-not-exists で冪等なため一定数の JVM タイミングエラーは許容）
WARN_THRESHOLD="${WARN_THRESHOLD:-5}"

# 各バックグラウンドジョブの PID を追跡する配列
declare -a PIDS=()

# CRIT-001 監査対応: トピック作成を並列度制限付きでバックグラウンド実行するヘルパー関数
# 実行中のバックグラウンドジョブが MAX_PARALLEL に達したら 1 つ完了するまで待機する
# --bootstrap-server は環境変数 BOOTSTRAP_SERVER から自動設定する
create_topic() {
  while [ "$(jobs -rp 2>/dev/null | wc -l || echo 0)" -ge "${MAX_PARALLEL}" ]; do
    sleep 0.1
  done
  kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" "$@" &
  PIDS+=($!)
}

echo "=== Creating Kafka topics (bootstrap: ${BOOTSTRAP_SERVER}) ==="

# --- System Tier ---
# 監査ログ (auth-server -> audit-aggregator)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.auth.audit.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=7776000000

# 設定変更通知 (config-server -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.config.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 認証ログイン (auth-server)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.auth.login.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 権限拒否 (auth-server -> audit)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.auth.permission_denied.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# APIレジストリ スキーマ更新 (api-registry -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.apiregistry.schema_updated.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# フィーチャーフラグ変更 (featureflag-server -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.featureflag.changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ファイルアップロード (file-server -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.file.uploaded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ファイル削除 (file-server -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.file.deleted.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ファイル汎用イベント（file-rust がプロデューサー、event_type ヘッダーで種別を区別）（M-21 監査対応）
# file-rust は topic_events 設定でこのトピックを使用する（uploaded.v1/deleted.v1 は将来のイベント分離用に残す）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.file.events.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# tenant イベントトピック（C-07 監査対応: データ損失防止のため追加）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.tenant.events.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# シークレットローテーション (vault-server -> subscribers)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.vault.secret_rotated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 通知リクエスト (notification-server -> delivery)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.notification.requested.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# クォータ超過 (quota-server -> alerting)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.quota.exceeded.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# Saga 状態変更 (saga-server -> orchestration)
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.saga.state_changed.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# トークン検証 (auth-server -> subscribers) ※topics.yaml k1s0.system.auth.token_validate.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.auth.token_validate.v1 \
  --partitions 6 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# マスタデータ変更 (mastermaintenance-server -> subscribers) ※topics.yaml k1s0.system.mastermaintenance.data_changed.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.mastermaintenance.data_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- Service Tier ---
# L-07 対応: topics.yaml との突合により task.updated.v1 / task.cancelled.v1 を追加する
create_topic \
  --create --if-not-exists \
  --topic k1s0.service.task.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# タスク更新イベント ※topics.yaml k1s0.service.task.updated.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.service.task.updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# タスクキャンセルイベント ※topics.yaml k1s0.service.task.cancelled.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.service.task.cancelled.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# board サービスのトピック
create_topic \
  --create --if-not-exists \
  --topic k1s0.service.board.column_updated.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# activity サービスのトピック
create_topic \
  --create --if-not-exists \
  --topic k1s0.service.activity.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

create_topic \
  --create --if-not-exists \
  --topic k1s0.service.activity.approved.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# --- Business Tier ---
# L-07 対応: topics.yaml との突合により business tier トピックを追加する
# プロジェクト種別変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.project_type_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ステータス定義変更イベント ※topics.yaml k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 と対応
create_topic \
  --create --if-not-exists \
  --topic k1s0.business.taskmanagement.projectmaster.status_definition_changed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# 検索インデックス更新イベント（search-rust がコンシューマー）（MED-1 監査対応）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.search.index.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# ワークフロー状態変更イベント（workflow-rust がプロデューサー）（MED-1 監査対応）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.workflow.state.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# スケジューライベント（scheduler-rust がプロデューサー）（M-20 監査対応: メイントピックが欠落していた）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.created.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# スケジューラ実行イベント（M-20 監査対応: メイントピックが欠落していた）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.executed.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# スケジューラトリガーイベント（M-20 監査対応: メイントピックが欠落していた）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.triggered.v1 \
  --partitions 3 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=604800000

# スケジューラ作成 DLQ（MED-1 監査対応: scheduler.created.v1 に対する DLQ が欠落していた）
create_topic \
  --create --if-not-exists \
  --topic k1s0.system.scheduler.created.v1.dlq \
  --partitions 1 \
  --replication-factor "${REPLICATION_FACTOR}" \
  --config retention.ms=2592000000

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
  create_topic \
    --create --if-not-exists \
    --topic "${topic}" \
    --partitions 1 \
    --replication-factor "${REPLICATION_FACTOR}" \
    --config retention.ms=2592000000
done

# CRIT-001 監査対応: 全バックグラウンドジョブの完了を待ち、失敗件数を確認する
# WARN_COUNT が WARN_THRESHOLD を超えた場合は OOM またはKafka 接続エラーと判断し exit 1 する
# docker-compose の restart: "on-failure:3" でリトライされ、--if-not-exists で冪等に再作成される
WARN_COUNT=0
for pid in "${PIDS[@]}"; do
  if ! wait "$pid"; then
    echo "WARN: PID $pid のトピック作成が非ゼロ終了（--if-not-exists 使用のため無視）" >&2
    WARN_COUNT=$((WARN_COUNT + 1))
  fi
done

if [ "${WARN_COUNT}" -gt "${WARN_THRESHOLD}" ]; then
  echo "ERROR: ${WARN_COUNT} 件のトピック作成が失敗しました（閾値: ${WARN_THRESHOLD}）" >&2
  echo "       OOM または Kafka 接続エラーの可能性があります。コンテナを再起動します。" >&2
  exit 1
elif [ "${WARN_COUNT}" -gt 0 ]; then
  echo "WARN: ${WARN_COUNT} 件のトピック作成が非ゼロ終了コードでした（冪等のため続行）" >&2
fi

echo "=== All Kafka topics created ==="

# CRIT-001 監査対応: トピック数を確認し、期待値未満の場合は失敗終了する
# docker-compose の restart: "on-failure:3" でリトライされる
# MED-005 監査対応: EXPECTED_TOPIC_COUNT を実際の作成数（58件）に修正する
#   - 内部トピック（__ プレフィックス）を grep -v で除外し k1s0 トピックのみをカウントする
#   - 旧値 57 は topics.yaml との不一致（58件）を修正する前の値
EXPECTED_TOPIC_COUNT="${EXPECTED_TOPIC_COUNT:-58}"
ACTUAL_COUNT=$(kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" --list 2>/dev/null \
  | grep -v '^__' \
  | wc -l || echo 0)
echo "トピック数確認: 作成済み ${ACTUAL_COUNT} / 期待値 ${EXPECTED_TOPIC_COUNT}"
if [ "${ACTUAL_COUNT}" -lt "${EXPECTED_TOPIC_COUNT}" ]; then
  echo "ERROR: トピック数が期待値に達していません（作成済み: ${ACTUAL_COUNT}, 期待値: ${EXPECTED_TOPIC_COUNT}）" >&2
  echo "       docker compose restart: on-failure:3 でリトライします。" >&2
  exit 1
fi

echo "=== Kafka topics verified successfully (${ACTUAL_COUNT} topics) ==="

# トピック一覧を表示
kafka-topics.sh --bootstrap-server "${BOOTSTRAP_SERVER}" --list
