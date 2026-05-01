---
runbook_id: RB-DB-002
title: PostgreSQL Primary 障害対応（CNPG failover）
category: DB
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: PostgresPrimaryDown
fmea_id: FMEA-006
estimated_recovery: 暫定 10 分 / 恒久 4 時間
last_updated: 2026-05-02
---

# RB-DB-002: PostgreSQL Primary 障害対応（CNPG failover）

本 Runbook は CloudNativePG が管理する `k1s0-postgres` クラスタの Primary が応答喪失した時の対応手順を定める。SLA 99% / NFR-A-CONT-001 / NFR-A-REC-002 / FMEA-006 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole を保持していること（`kubectl auth can-i get cluster.postgresql.cnpg.io --all-namespaces` で確認）。
- 必要ツール: `kubectl` (>=1.30) / `kubectl cnpg` プラグイン / `psql` クライアント。
- kubectl context が本番（`kubectl config current-context` が `k1s0-prod`）であること。staging で誤実行しないこと。
- CNPG Operator が起動済（`kubectl get deploy -n cnpg-system cnpg-controller-manager` が `1/1`）であること。Operator が落ちている場合は本 Runbook では復旧不可、Operator 起動を先行すること。

## 2. 対象事象

- Alertmanager `PostgresPrimaryDown` 発火（`cnpg_pg_up{role="primary"} == 0` を 60 秒継続）、または
- `kubectl get cluster k1s0-postgres -n cnpg-system` で `STATUS=Unhealthy` かつ `currentPrimary` が空 / 古い Pod 名のまま停止、または
- tier1 facade のログに `connection refused` / `FATAL: database system is starting up` が連続出力。

検知シグナル:

```promql
# primary が応答しているか（0 = 応答なし）
cnpg_pg_up{namespace="cnpg-system", cluster="k1s0-postgres", role="primary"} == 0

# WAL 送信ラグ（standby が古いまま滞留、failover 直前のサイン）
cnpg_pg_replication_lag{cluster="k1s0-postgres"} > 30
```

ダッシュボード: **Grafana → k1s0 PostgreSQL Overview**（`infra/observability/grafana/` に定義）。
通知経路: PagerDuty `tier1-platform-team` → Slack `#incident-sev1`。

## 3. 初動手順（5 分以内）

最初の 5 分でクラスタ状態を把握し、自動 failover が動作しているか判定する。

```bash
# CNPG Cluster の現状確認
kubectl get cluster k1s0-postgres -n cnpg-system -o wide
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres
```

```bash
# primary Pod のログ確認（既知 Primary が分かる場合）
PRIM=$(kubectl get cluster k1s0-postgres -n cnpg-system \
  -o jsonpath='{.status.currentPrimary}')
kubectl logs -n cnpg-system "${PRIM}" --tail=100
```

```bash
# Node 障害か Pod 障害かを切り分ける
kubectl get node $(kubectl get pod -n cnpg-system "${PRIM}" \
  -o jsonpath='{.spec.nodeName}') -o wide
```

```bash
# CNPG が自動 failover を開始しているか確認
kubectl describe cluster k1s0-postgres -n cnpg-system | grep -A5 "Current Primary"
```

```bash
# tier1 facade が DB 接続エラーを返しているか確認
kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -i "pgsql\|connection\|FATAL"
```

ステークホルダー通知: Slack `#status` に「PostgreSQL Primary 応答喪失、CNPG 自動 failover 待機中」を 5 分以内に投稿。SEV1 確定なら `RB-INC-001`（severity-decision-tree）と `oncall/escalation.md` を起動する。

## 4. 原因特定手順

ログとイベントから根本原因を特定する。原因が特定できなくても 5. 復旧手順に進んで業務影響を止めることが優先。

```bash
# CNPG operator ログ
kubectl logs -n cnpg-system deploy/cnpg-controller-manager --tail=200 | grep -i "k1s0-postgres"

# PostgreSQL クラッシュ原因
kubectl logs -n cnpg-system "${PRIM}" --previous | tail -100

# Pod イベント
kubectl describe pod "${PRIM}" -n cnpg-system | tail -40
```

よくある原因:

1. **OOM Kill**: Node の memory pressure で Pod が kill された。`kubectl describe pod <prim> -n cnpg-system` で `OOMKilled` を確認。対処: メモリ Limit を引き上げる（`infra/data/cloudnativepg/cluster.yaml`）。
2. **PVC フル**: WAL ファイルが蓄積して `/var` が枯渇。`kubectl exec <pod> -- df -h` で確認。対処: WAL アーカイブを MinIO に手動フラッシュ後、古い WAL を削除。
3. **Node 障害**: Worker Node の hardware fault。`kubectl get events -n cnpg-system` と Cloud Provider コンソールを確認。
4. **ネットワーク分断（Split-Brain）**: CNPG はフェンシングで旧 primary をロックするが、`cnpg.io/fenced=all` アノテーションが残留していないか確認する。
5. **Barman アーカイブ失敗**: MinIO が停止し WAL アーカイブが詰まった場合、primary が新規接続を拒否する。`kubectl logs <pod> | grep barman` で確認。MinIO 障害が原因なら `RB-BKP-001` を並行起動する。

エスカレーション: 原因が上記 5 パターンに該当しない、または Operator ログが解読できない場合は L3 起案者へ Slack で連絡。

## 5. 復旧手順

CNPG 自動 failover（`primaryUpdateStrategy: unsupervised`）が動作している場合:

```bash
# CNPG は 30s 以内に standby を primary に昇格させる。
# Cluster の status.currentPrimary が切り替わったことを確認する。
kubectl get cluster k1s0-postgres -n cnpg-system -w
```

自動 failover が起動しない場合（手動 failover）:

```bash
# standby 一覧を確認
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres,role=replica

# 最新 LSN の standby を選択して手動昇格
kubectl cnpg promote k1s0-postgres -n cnpg-system --instance k1s0-postgres-2
```

failover 後の接続復旧:

```bash
# 新 primary で接続確認
kubectl exec -n cnpg-system k1s0-postgres-2 -- psql -U k1s0 -c "SELECT pg_is_in_recovery();"
# → f が返れば primary として稼働中

# tier1 facade の rolling restart（接続プールのリセット）
kubectl rollout restart deployment/tier1-facade -n k1s0
kubectl rollout status deployment/tier1-facade -n k1s0
```

旧 primary の復旧（Node が生き返った場合）:

```bash
# CNPG が自動的に旧 primary を standby として再参加させる
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres -w
```

## 6. 検証手順

復旧完了の判定基準。全項目が満たされてはじめてインシデントを Resolved に遷移できる。

- 新 Primary が `SELECT pg_is_in_recovery()` で `f` を返す。
- `cnpg_pg_up{role="primary"} == 1` が 5 分間継続して観測される（Grafana → k1s0 PostgreSQL Overview）。
- standby 2 台の `cnpg_pg_replication_lag` が 1 秒以内に収束。
- tier1 facade の `/healthz` が 200 を返し、`kubectl logs deploy/tier1-facade -n k1s0 --tail=50 | grep ERROR` で DB 接続エラーが新規発生していない。
- 直近 10 分の Loki クエリ `{namespace="cnpg-system"} |= "FATAL"` でエラーが 0 件。
- Barman backup が次回スケジュール内（通常 6h 以内）で成功している（`kubectl cnpg status k1s0-postgres -n cnpg-system | grep "Last successful backup"`）。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-RB-DB-002.md`）。
- Mimir alert ルールの閾値チューニング（誤検知であれば `replication_lag` 閾値を調整）。
- MinIO バックアップが欠落していないか確認（`kubectl cnpg status k1s0-postgres -n cnpg-system`）。
- PDB（PodDisruptionBudget）の設定が適切か再確認。`infra/data/cloudnativepg/cluster.yaml` の `affinity.podAntiAffinity` で 3 台が異なる Node に配置されているか確認。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 10 分以内、恒久 4 時間以内）。
- 月次 Chaos Drill 対象に本 Runbook を含める（`ops/chaos/workflows/monthly-game-day.yaml`）。

## 8. 関連 Runbook

- 関連設計書: `infra/data/cloudnativepg/cluster.yaml`
- 関連 ADR: [ADR-DATA-001](../../../docs/02_構想設計/adr/ADR-DATA-001-cnpg.md)
- 関連 NFR: [NFR-A-CONT-001 / NFR-A-REC-002](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: [FMEA-006](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — SPIRE が CNPG に依存しているため証明書期限切れと連鎖する
  - [`RB-BKP-001-backup-failure.md`](RB-BKP-001-backup-failure.md) — MinIO に Barman アーカイブが向かうため、Backup 失敗と原因が共通する
  - [`RB-DR-001-cluster-rebuild.md`](../../dr/scenarios/RB-DR-001-cluster-rebuild.md) — Primary が回復不能な場合に DR 経路へ移行
