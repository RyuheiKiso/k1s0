# PostgreSQL Primary 応答喪失 Runbook

> **alert_id**: tier1.postgres.availability.primary-down
> **severity**: SEV1
> **owner**: tier1-platform-team
> **estimated_mttr**: 20m
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

**Mimir / Grafana** で以下を確認する。

PromQL（Mimir）:

```promql
# primary が応答しているか（0 = 応答なし）
cnpg_pg_up{namespace="cnpg-system", cluster="k1s0-postgres", role="primary"} == 0

# WAL 送信ラグ（standby が古いまま滞留）
cnpg_pg_replication_lag{cluster="k1s0-postgres"} > 30
```

ダッシュボード: **Grafana → k1s0 PostgreSQL Overview**（`infra/observability/grafana/` に定義）。

alert チャンネル: PagerDuty `tier1-platform-team` → Slack `#incident-sev1`。

## 2. 初動 (Immediate Action, 〜15 分)

- [ ] CNPG Cluster の現状確認

  ```bash
  kubectl get cluster k1s0-postgres -n cnpg-system -o wide
  kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres
  ```

- [ ] primary Pod のログ確認

  ```bash
  PRIM=$(kubectl get cluster k1s0-postgres -n cnpg-system \
    -o jsonpath='{.status.currentPrimary}')
  kubectl logs -n cnpg-system "${PRIM}" --tail=100
  ```

- [ ] Node 障害か Pod 障害かを切り分ける

  ```bash
  kubectl get node $(kubectl get pod -n cnpg-system "${PRIM}" \
    -o jsonpath='{.spec.nodeName}') -o wide
  ```

- [ ] CNPG が自動フェイルオーバーを開始しているか確認

  ```bash
  kubectl describe cluster k1s0-postgres -n cnpg-system | grep -A5 "Current Primary"
  ```

- [ ] tier1 facade が DB 接続エラーを返しているか確認

  ```bash
  kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -i "pgsql\|connection\|FATAL"
  ```

## 3. 復旧 (Recovery, 〜60 分)

**CNPG 自動フェイルオーバー（primaryUpdateStrategy: unsupervised）が動作している場合**:

CNPG は `30s` 以内に standby を primary に昇格させる。Cluster の `status.currentPrimary` が切り替わったことを確認する。

```bash
kubectl get cluster k1s0-postgres -n cnpg-system -w
```

**自動フェイルオーバーが起動しない場合（手動フェイルオーバー）**:

```bash
# standby 一覧を確認
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres,role=replica

# 最新 LSN の standby を選択して手動昇格
kubectl cnpg promote k1s0-postgres -n cnpg-system --instance k1s0-postgres-2
```

**フェイルオーバー後の確認**:

```bash
# 新 primary で接続確認
kubectl exec -n cnpg-system k1s0-postgres-2 -- psql -U k1s0 -c "SELECT pg_is_in_recovery();"
# → f が返れば primary として稼働中

# tier1 facade の rolling restart（接続プールのリセット）
kubectl rollout restart deployment/tier1-facade -n k1s0
kubectl rollout status deployment/tier1-facade -n k1s0
```

**旧 primary の復旧（Node が生き返った場合）**:

```bash
# CNPG が自動的に旧 primary を standby として再参加させる
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-postgres -w
```

## 4. 原因調査 (Root Cause Analysis)

**ログ確認**:

```bash
# CNPG operator ログ
kubectl logs -n cnpg-system deploy/cnpg-controller-manager --tail=200 | grep -i "k1s0-postgres"

# PostgreSQL クラッシュ原因
kubectl logs -n cnpg-system "${PRIM}" --previous | tail -100
```

**よくある原因**:

1. **OOM Kill**: Node の memory pressure で Pod が kill された。`kubectl describe pod <prim> -n cnpg-system` で `OOMKilled` を確認。対処: メモリ Limit を引き上げる（`infra/data/cloudnativepg/cluster.yaml`）。
2. **PVC フル**: WAL ファイルが蓄積して `/var` が枯渇。`kubectl exec <pod> -- df -h` で確認。対処: WAL アーカイブを MinIO に手動フラッシュ後、古い WAL を削除。
3. **Node 障害**: Worker Node の hardware fault。`kubectl get events -n cnpg-system` と Cloud Provider コンソールを確認。
4. **ネットワーク分断（Split-Brain）**: CNPG はフェンシングで旧 primary をロックするが、`cnpg.io/fenced=all` アノテーションが残留していないか確認する。
5. **Barman アーカイブ失敗**: MinIO が停止し WAL アーカイブが詰まった場合、primary が新規接続を拒否する。`kubectl logs <pod> | grep barman` で確認。

## 5. 事後処理 (Post-incident)

- [ ] ポストモーテム起票（24 時間以内、`ops/runbooks/postmortems/<YYYY-MM-DD>-postgres-primary-down.md`）
- [ ] Mimir alert ルールの閾値チューニング（誤検知であれば `replication_lag` 閾値を調整）
- [ ] MinIO バックアップが欠落していないか確認（`kubectl cnpg status k1s0-postgres -n cnpg-system`）
- [ ] PDB（PodDisruptionBudget）の設定が適切か再確認
- [ ] NFR-A-REC-002 の MTTR ログを更新

## 関連

- 関連設計書: `infra/data/cloudnativepg/cluster.yaml`
- 関連 ADR: `docs/02_構想設計/adr/ADR-DATA-001`
- 関連 Runbook: `ops/runbooks/incidents/mtls-cert-expiry.md`（SPIRE が CNPG に依存）
