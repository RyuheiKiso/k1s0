---
runbook_id: RB-MSG-001
title: Kafka ブローカー障害対応
category: MSG
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: KafkaBrokerDown
fmea_id: FMEA-003
estimated_recovery: 暫定 15 分 / 恒久 4 時間
last_updated: 2026-05-02
---

# RB-MSG-001: Kafka ブローカー障害対応

本 Runbook は Strimzi が管理する `k1s0-kafka` クラスタの broker が 1 台障害となった時の対応を定める。3 broker 構成で 1 台喪失は SEV2（継続稼働可能）、2 台同時喪失で SEV1 に昇格。NFR-A-CONT-001 / NFR-C-MON-002 / FMEA-003 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole を保持し、`kafka` namespace の Pod exec / log 取得権限を持つこと。
- 必要ツール: `kubectl` (>=1.30) / `kafka-topics.sh` 等の bundled CLI（broker Pod 内で実行）。
- kubectl context が `k1s0-prod`。
- Strimzi Cluster Operator が起動済（`kubectl get deploy -n kafka strimzi-cluster-operator` が `1/1`）。Operator が落ちている場合は本 Runbook では復旧不可、Operator 起動を先行すること。
- `infra/data/kafka/kafka-cluster.yaml` の `replication.factor=3` / `min.insync.replicas=2` 構成を前提とする（崩れていると本手順は適用不可）。

## 2. 対象事象

- Alertmanager `KafkaBrokerDown` 発火（`kafka_server_replicamanager_offlinereplicacount > 0` を 60 秒継続）、または
- `kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka` で 1 Pod が `CrashLoopBackOff` / `Pending` / `Unknown`、または
- tier1 facade の Dapr pubsub publish が `failed to publish to kafka: broker not available` エラーを返す。

検知シグナル:

```promql
# Broker がオフラインになった数（1 以上でアラート）
kafka_server_replicamanager_offlinereplicacount{namespace="kafka"} > 0

# Under-replicated partition 数（broker 障害時に増加）
kafka_server_replicamanager_underreplicatedpartitions{namespace="kafka"} > 0

# 稼働中の broker 数（3 台構成なので 2 未満で SEV1 昇格）
count(up{job="kafka-metrics", namespace="kafka"}) < 3
```

ダッシュボード: **Grafana → k1s0 Kafka Overview**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#incident-kafka`。

## 3. 初動手順（5 分以内）

最初の 5 分でクラスタ構成と quorum 健全性を確認し、SEV2 で継続可能か / SEV1 昇格が必要かを判定する。

```bash
# Kafka Cluster の状態確認
kubectl get kafka k1s0-kafka -n kafka -o wide
kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka
```

```bash
# 障害 broker Pod のログ確認（例: dual-role-1 が障害の場合）
kubectl logs -n kafka k1s0-kafka-dual-role-1 --tail=100
kubectl describe pod k1s0-kafka-dual-role-1 -n kafka
```

```bash
# KRaft metadata quorum の健全性確認（3 台中 2 台生存で quorum 維持）
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-metadata-quorum.sh --bootstrap-server localhost:9093 describe --status
```

```bash
# Under-replicated partitions の一覧確認
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9093 \
  --describe --under-replicated-partitions
```

```bash
# Dapr pubsub component（infra/dapr/components/pubsub/kafka.yaml）経由の publish が失敗していないか確認
kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -i "kafka\|pubsub"
```

ステークホルダー通知: 1 台喪失で継続稼働中なら Slack `#status` に「Kafka 1 broker 喪失、継続稼働中」を投稿。2 台同時喪失なら SEV1 に昇格して `oncall/escalation.md` を起動する。

## 4. 原因特定手順

```bash
# Strimzi Cluster Operator ログ
kubectl logs -n kafka deploy/strimzi-cluster-operator --tail=200 | grep "k1s0-kafka"

# JVM OOM の確認
kubectl logs -n kafka k1s0-kafka-dual-role-1 --previous | grep -i "OutOfMemory\|killed"
```

よくある原因:

1. **PVC 容量不足**: `log.dirs` が満杯になり broker が自動停止。`kubectl exec <pod> -- df -h /var/kafka-log` で確認。対処: `log.retention.hours` の短縮または PVC 拡張。
2. **JVM Heap 不足（OOM Kill）**: 4Gi Limit に対して GC が追いつかない。`kubectl top pod -n kafka` で確認。対処: `spec.resources.limits.memory` を引き上げる。
3. **KRaft quorum 割れ（2 台同時障害）**: `kafka-metadata-quorum.sh` でリーダーが選出されていないことを確認。3 台中 2 台復旧後に自動回復する。SEV1 昇格対象。
4. **TLS 証明書期限切れ**: mTLS listener（port 9093）の証明書が失効。`kubectl get secret -n kafka | grep cluster-ca` で確認。対処: [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) を参照。
5. **Node 障害 / eviction**: PodAntiAffinity で分散配置済みだが同一 rack 障害の場合に複数 broker が同時ダウンする。

エスカレーション: KRaft quorum 割れ（2 台喪失）の場合は L3 起案者へ即時連絡し SEV1 昇格。

## 5. 復旧手順

Strimzi による自動 Pod 再起動を待つ（通常は 2〜5 分）:

```bash
kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka -w
```

自動再起動が起きない場合（Pod が Pending / CrashLoopBackOff）:

```bash
# Pod を強制削除して再スケジュール
kubectl delete pod k1s0-kafka-dual-role-1 -n kafka

# PVC が残っているか確認（PVC が消えると broker は空で再参加）
kubectl get pvc -n kafka -l strimzi.io/cluster=k1s0-kafka
```

Node 障害で Pod が再スケジュールできない場合:

```bash
# Node を drain して別 Node へ追い出す
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data
# Strimzi が自動で pod を新 Node に配置する
```

Partition 再同期の確認:

```bash
# Under-replicated が 0 になるまで待機（通常 5〜15 分）
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9093 \
  --describe --under-replicated-partitions
```

Cruise Control でリバランス（不均衡が大きい場合）:

```bash
kubectl annotate kafkarebalance k1s0-kafka-rebalance \
  strimzi.io/rebalance=approve -n kafka --overwrite
```

## 6. 検証手順

復旧完了の判定基準:

- 全 3 broker が `Running` かつ `Ready=True`（`kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka` で確認）。
- `kafka_server_replicamanager_offlinereplicacount == 0` が 5 分間継続。
- `kafka_server_replicamanager_underreplicatedpartitions == 0`（partition 再同期完了）。
- KRaft quorum が 3/3 healthy（`kafka-metadata-quorum.sh describe --status` で `LeaderId` が確認できる）。
- tier1 facade の Dapr pubsub publish が新規エラーなし（直近 10 分の Loki クエリ `{namespace="k1s0"} |= "kafka" |= "ERROR"` が 0 件）。
- DLQ 滞留がない（`kafka_consumergroup_lag{topic=~".*\\.dlq"} == 0`、滞留時は `RB-MSG-002` に連鎖）。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-RB-MSG-001.md`）。
- Cruise Control のリバランス結果をダッシュボードで確認し、partition 配置の偏りを是正。
- `log.retention.hours` または PVC サイズの見直し（ディスク不足起因の場合）。
- DLQ にメッセージが滞留していないか確認（[`RB-MSG-002`](RB-MSG-002-dlq-backlog.md)、暫定名 — 旧 `dlq-backlog.md`）。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 15 分以内、恒久 4 時間以内）。
- 月次 Chaos Drill 対象に本 Runbook を含める（`ops/chaos/experiments/pod-delete/kafka-broker.yaml`）。

## 8. 関連 Runbook

- 関連設計書: `infra/data/kafka/kafka-cluster.yaml`
- 関連 ADR: [ADR-DATA-002](../../../docs/02_構想設計/adr/ADR-DATA-002-kafka.md)
- 関連 NFR: [NFR-A-CONT-001 / NFR-C-MON-002](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: [FMEA-003](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-MSG-002-dlq-backlog.md`](RB-MSG-002-dlq-backlog.md) — DLQ 滞留が同時発生した場合
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — mTLS 証明書期限切れが原因の場合
  - [`RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) — 全 broker 喪失時に DR 経路へ
