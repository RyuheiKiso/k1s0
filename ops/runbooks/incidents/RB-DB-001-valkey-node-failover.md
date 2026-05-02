---
runbook_id: RB-DB-001
title: Valkey クラスタノード障害対応（Sentinel 自動 failover）
category: DB
severity: SEV3
owner: 協力者
automation: manual
alertmanager_rule: ValkeyNodeDown
fmea_id: FMEA-007
estimated_recovery: 暫定 10 分 / 恒久 2 時間
last_updated: 2026-05-02
---

# RB-DB-001: Valkey クラスタノード障害対応（Sentinel 自動 failover）

本 Runbook は Valkey Cluster（State Store / セッションキャッシュ / 分散ロック用）の master または replica が 1 ノード障害となった時の対応を定める。
3 master + 3 replica の 6 ノード構成のため 1 ノード喪失は SEV3（自動 failover で継続稼働可能）、複数ノード同時喪失で SEV1 昇格。
NFR-A-CONT-001 / FMEA-007 / ADR-DATA-004 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole を保持し、`k1s0-data` namespace の Pod exec / log 権限を持つこと。
- 必要ツール: `kubectl` (>=1.30) / `valkey-cli`（master Pod 内で実行）/ `bao`（パスワード取得用）。
- kubectl context が `k1s0-prod`。
- Valkey Helm release 名は `valkey`、namespace `k1s0-data`。
- `infra/data/valkey/values.yaml` の `cluster.nodes=6`（master 3 + replica 3）構成を前提とする。

## 2. 対象事象

- Alertmanager `ValkeyNodeDown` 発火（`up{job="valkey",namespace="k1s0-data"} == 0` を 60 秒継続）、または
- `kubectl get pods -n k1s0-data -l app.kubernetes.io/name=valkey-cluster` で 1 Pod が `CrashLoopBackOff` / `Pending` / `Unknown`、または
- tier1 facade の State API が `redis: connection refused` を返す。

検知シグナル:

```promql
# Valkey ノード稼働状態
up{namespace="k1s0-data", job="valkey-cluster"} == 0

# Cluster の slot カバレッジ（全 16384 slots がカバーされているか）
valkey_cluster_slots_assigned < 16384

# Replication lag（replica が遅れている兆候）
valkey_master_repl_offset - valkey_slave_repl_offset > 1000000
```

ダッシュボード: **Grafana → k1s0 Valkey Cluster**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#alert-data`。

## 3. 初動手順（5 分以内）

```bash
# Cluster 状態確認
kubectl get pods -n k1s0-data -l app.kubernetes.io/name=valkey-cluster -o wide
```

```bash
# 障害 Pod のログ確認（例: valkey-cluster-1 が障害）
kubectl logs -n k1s0-data valkey-cluster-1 --tail=100
kubectl describe pod valkey-cluster-1 -n k1s0-data
```

```bash
# Valkey CLI で cluster 状態確認（生存している master pod から）
PASS=$(kubectl get secret -n k1s0-data k1s0-valkey-credentials -o jsonpath='{.data.password}' | base64 -d)
kubectl exec -n k1s0-data valkey-cluster-0 -- \
  valkey-cli -a "${PASS}" --tls cluster info
```

```bash
# 各ノードの role と slave 状態
kubectl exec -n k1s0-data valkey-cluster-0 -- \
  valkey-cli -a "${PASS}" --tls cluster nodes | head -10
```

```bash
# tier1 facade の Valkey 接続エラー確認
kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -iE "valkey|redis|connection"
```

ステークホルダー通知: 1 ノード喪失は SEV3 のため `#alert-data` に「Valkey 1 node down、自動 failover 進行中」を投稿。
2 ノード以上同時喪失 / cluster slot 部分喪失なら SEV1 昇格して `oncall/escalation.md` 起動。

## 4. 原因特定手順

```bash
# Pod イベント確認
kubectl get events -n k1s0-data --sort-by='.lastTimestamp' | head -20

# OOM Kill 確認
kubectl logs -n k1s0-data valkey-cluster-1 --previous | grep -iE "OutOfMemory|killed"

# PVC 容量確認
kubectl exec -n k1s0-data valkey-cluster-1 -- df -h /bitnami/valkey/data
```

よくある原因:

1. **OOM Kill**: メモリ Limit に対してデータ量が膨張。Persistence ファイルの肥大化を確認。対処: `maxmemory-policy: allkeys-lru` 確認、Limit 引き上げ。
2. **PVC フル**: AOF / RDB ファイルがディスクを圧迫。`valkey-cli BGREWRITEAOF` でコンパクション。
3. **Persistence ファイル破損**: 起動時に `Bad file format reading the append only file` エラー。対処: 該当 Pod の PVC をクリアして再 sync。
4. **Node 障害**: Worker Node の hardware fault。PodAntiAffinity で分散配置されているか確認。
5. **TLS 証明書期限切れ**: cert-manager の証明書失効で TLS handshake 失敗。[`RB-SEC-002`](RB-SEC-002-cert-expiry.md) 並行起動。

エスカレーション: 複数ノード同時障害（cluster quorum 喪失）の場合は L3 起案者へ連絡し SEV1 昇格。

## 5. 復旧手順

### 自動 failover を待つ（〜2 分）

Sentinel または cluster ノード間の gossip プロトコルが master 喪失を検知すると、replica が自動 promote される:

```bash
kubectl get pods -n k1s0-data -l app.kubernetes.io/name=valkey-cluster -w
```

Cluster info で role 切替を確認:

```bash
kubectl exec -n k1s0-data valkey-cluster-0 -- \
  valkey-cli -a "${PASS}" --tls cluster nodes | grep master
```

### 自動 failover が動作しない場合（手動 failover）

```bash
# replica Pod から手動 promote
kubectl exec -n k1s0-data valkey-cluster-3 -- \
  valkey-cli -a "${PASS}" --tls cluster failover takeover
```

### Pod が CrashLoopBackOff / Pending の場合

```bash
# Pod を強制削除して再スケジュール
kubectl delete pod valkey-cluster-1 -n k1s0-data

# PVC が壊れている場合は PVC ごと削除（データ喪失するが他 master + replica で補完）
kubectl delete pvc data-valkey-cluster-1 -n k1s0-data
kubectl delete pod valkey-cluster-1 -n k1s0-data
# StatefulSet が新 PVC で Pod を再作成、cluster meet で復帰
```

### Cluster meet で再参加

新 Pod が cluster に再参加しない場合:

```bash
NEW_IP=$(kubectl get pod valkey-cluster-1 -n k1s0-data -o jsonpath='{.status.podIP}')
kubectl exec -n k1s0-data valkey-cluster-0 -- \
  valkey-cli -a "${PASS}" --tls cluster meet "${NEW_IP}" 6379
```

### tier1 facade の接続プールリセット

failover 後、古い master IP に接続が残っている場合:

```bash
kubectl rollout restart deployment/tier1-facade -n k1s0
kubectl rollout status deployment/tier1-facade -n k1s0
```

## 6. 検証手順

復旧完了の判定基準:

- 全 6 ノードが `Running` かつ `Ready=True`（`kubectl get pods -n k1s0-data -l app.kubernetes.io/name=valkey-cluster`）。
- Cluster slot coverage が 16384/16384（`valkey-cli cluster info` で `cluster_slots_assigned:16384`）。
- Replication lag が 0 もしくは < 100ms（`valkey_master_repl_offset - valkey_slave_repl_offset` を Grafana で確認）。
- master 数が 3、replica 数が 3（`cluster nodes` で確認）。
- tier1 facade の State API が `/healthz` 200、Loki クエリ `{namespace="k1s0"} |= "valkey" |= "ERROR"` が直近 10 分で 0 件。
- `up{job="valkey-cluster"} == 1` が全ノードで 5 分間継続。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-DB-001.md`）。
- メモリ Limit / `maxmemory-policy` の見直し（OOM 起因の場合）。
- PVC サイズ拡張（ディスク不足起因の場合、`infra/data/valkey/values.yaml` の `persistence.size` 更新）。
- PodAntiAffinity の確認（`infra/data/valkey/values.yaml` の `affinity` セクション）— 同一 Node に複数 master が配置されないよう topology key を確認。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 10 分以内）。
- 月次 Chaos Drill 対象に「Valkey master Pod kill」シナリオを追加（`ops/chaos/experiments/pod-delete/valkey-master.yaml`）。

## 8. 関連 Runbook

- 関連設計書: `infra/data/valkey/values.yaml`、[`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/05_データ層配置.md`](../../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/05_データ層配置.md)
- 関連 ADR: [ADR-DATA-004（Valkey 採用）](../../../docs/02_構想設計/adr/ADR-DATA-004-valkey.md)
- 関連 NFR: [NFR-A-CONT-001](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: [FMEA-007](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — TLS 証明書期限切れが原因の場合
  - [`RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) — Cluster quorum 完全喪失時
