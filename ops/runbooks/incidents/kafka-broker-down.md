# Kafka Broker 障害 Runbook

> **alert_id**: tier1.kafka.availability.broker-down
> **severity**: SEV2
> **owner**: tier1-platform-team
> **estimated_mttr**: 30m
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する。

PromQL（Mimir）:

```promql
# Broker がオフラインになった数（1 以上でアラート）
kafka_server_replicamanager_offlinereplicacount{namespace="kafka"} > 0

# Under-replicated partition 数（broker 障害時に増加）
kafka_server_replicamanager_underreplicatedpartitions{namespace="kafka"} > 0

# 稼働中の broker 数（3 台構成なので 2 未満で SEV1 昇格）
count(up{job="kafka-metrics", namespace="kafka"}) < 3
```

ダッシュボード: **Grafana → k1s0 Kafka Overview**。

alert チャンネル: PagerDuty `tier1-platform-team` → Slack `#incident-kafka`。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] Kafka Cluster の状態確認

  ```bash
  kubectl get kafka k1s0-kafka -n kafka -o wide
  kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka
  ```

- [ ] 障害 broker Pod のログ確認

  ```bash
  # 例: dual-role-1 が障害の場合
  kubectl logs -n kafka k1s0-kafka-dual-role-1 --tail=100
  kubectl describe pod k1s0-kafka-dual-role-1 -n kafka
  ```

- [ ] KRaft metadata quorum の健全性確認（3 台中 2 台生存で quorum 維持）

  ```bash
  kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
    bin/kafka-metadata-quorum.sh --bootstrap-server localhost:9093 describe --status
  ```

- [ ] Under-replicated partitions の一覧確認

  ```bash
  kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
    bin/kafka-topics.sh --bootstrap-server localhost:9093 \
    --describe --under-replicated-partitions
  ```

- [ ] Dapr pubsub component（`infra/dapr/components/pubsub/kafka.yaml`）経由の publish が失敗していないか確認

  ```bash
  kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -i "kafka\|pubsub"
  ```

## 3. 復旧 (Recovery, 〜60 分)

**Strimzi による自動 Pod 再起動を待つ（通常は 2〜5 分）**:

```bash
kubectl get pods -n kafka -l strimzi.io/cluster=k1s0-kafka -w
```

**自動再起動が起きない場合（Pod が Pending / CrashLoopBackOff）**:

```bash
# Pod を強制削除して再スケジュール
kubectl delete pod k1s0-kafka-dual-role-1 -n kafka

# PVC が残っているか確認（PVC が消えると broker は空で再参加）
kubectl get pvc -n kafka -l strimzi.io/cluster=k1s0-kafka
```

**Node 障害で Pod が再スケジュールできない場合**:

```bash
# Node を drain して別 Node へ追い出す
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data
# Strimzi が自動で pod を新 Node に配置する
```

**Partition 再同期の確認**:

```bash
# Under-replicated が 0 になるまで待機（通常 5〜15 分）
kubectl exec -n kafka k1s0-kafka-dual-role-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9093 \
  --describe --under-replicated-partitions
```

**Cruise Control でリバランス（不均衡が大きい場合）**:

```bash
kubectl annotate kafkarebalance k1s0-kafka-rebalance \
  strimzi.io/rebalance=approve -n kafka --overwrite
```

## 4. 原因調査 (Root Cause Analysis)

**ログ確認**:

```bash
# Strimzi Cluster Operator ログ
kubectl logs -n kafka deploy/strimzi-cluster-operator --tail=200 | grep "k1s0-kafka"

# JVM OOM の確認
kubectl logs -n kafka k1s0-kafka-dual-role-1 --previous | grep -i "OutOfMemory\|killed"
```

**よくある原因**:

1. **PVC 容量不足**: `log.dirs` が満杯になり broker が自動停止。`kubectl exec <pod> -- df -h /var/kafka-log` で確認。対処: `log.retention.hours` の短縮または PVC 拡張。
2. **JVM Heap 不足（OOM Kill）**: 4Gi Limit に対して GC が追いつかない。`kubectl top pod -n kafka` で確認。対処: `spec.resources.limits.memory` を引き上げる。
3. **KRaft quorum 割れ（2 台同時障害）**: `kafka-metadata-quorum.sh` でリーダーが選出されていないことを確認。3 台中 2 台復旧後に自動回復する。
4. **TLS 証明書期限切れ**: mTLS listener（port 9093）の証明書が失効。`kubectl get secret -n kafka | grep cluster-ca` で確認。対処: `ops/runbooks/incidents/mtls-cert-expiry.md` を参照。
5. **Node 障害 / eviction**: PodAntiAffinity で分散配置済みだが同一 rack 障害の場合に複数 broker が同時ダウンする。

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-kafka-broker-down.md`）
- [ ] Cruise Control のリバランス結果をダッシュボードで確認
- [ ] `log.retention.hours` または PVC サイズの見直し（ディスク不足起因の場合）
- [ ] DLQ にメッセージが滞留していないか確認（`ops/runbooks/incidents/dlq-backlog.md`）
- [ ] NFR-A-REC-002 の MTTR ログを更新

## 関連

- 関連設計書: `infra/data/kafka/kafka-cluster.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-DATA-002`
- 関連 Runbook: `ops/runbooks/incidents/dlq-backlog.md`
