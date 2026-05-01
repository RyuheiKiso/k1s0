---
runbook_id: RB-SEC-001
title: OpenBao Raft リーダ選出失敗対応
category: SEC
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: OpenBaoNoLeader
fmea_id: FMEA-002
estimated_recovery: 暫定 30 分（手動介入）/ 恒久 2 時間
last_updated: 2026-05-02
---

# RB-SEC-001: OpenBao Raft リーダ選出失敗対応

本 Runbook は OpenBao クラスタで Raft リーダ選出が失敗した（Quorum 喪失 / Sealed 状態）時の対応を定める。
OpenBao 全停止は tier1 全 API の Secret 取得不可となり SEV1 全停止。
NFR-E-AC-001 / NFR-E-AC-004 / FMEA-002 / ADR-SEC-002 に対応する。

## 1. 前提条件

- 実行者は `security-sre` ClusterRole + OpenBao root token（または unseal share の保有者）。
- 必要ツール: `kubectl` / `bao`（OpenBao CLI）。
- kubectl context が `k1s0-prod`。
- OpenBao Helm release 名は `openbao`、namespace `openbao`。
- `infra/security/openbao/values.yaml` の HA + Raft integrated storage（3 replica）構成を前提。
- Shamir 5/3 構成の unseal share 保有者（最低 3 名）が連絡可能であること（[`oncall/contacts.md`](../../oncall/contacts.md)）。

## 2. 対象事象

- Alertmanager `OpenBaoNoLeader` 発火（OpenBao の `/v1/sys/leader` が `is_self=false` かつ `leader_address=""` を 60 秒継続）、または
- tier1 facade の Secret 取得エラー急増（`{app="tier1-facade"} |= "secret" |= "5"` Loki クエリ）、または
- `kubectl get pods -n openbao` で全 Pod が `Running` でも `bao status` が `Initialized: true, Sealed: true`、または
- Raft クラスタの過半数（2/3 以上）喪失。

検知シグナル:

```promql
# OpenBao の sealed 状態（1 = sealed、tier1 全 API 影響）
openbao_core_unsealed{namespace="openbao"} == 0

# Raft cluster の health（leader 不在）
openbao_raft_leader_lease_seconds{namespace="openbao"} == 0

# tier1 facade の Secret 取得エラー率
sum(rate(tier1_secret_fetch_errors_total[5m])) > 0.1
```

ダッシュボード: **Grafana → k1s0 OpenBao Cluster**。
通知経路: PagerDuty `security-sre` → Slack `#incident-sev1` → CTO 即時連絡。

## 3. 初動手順（5 分以内）

```bash
# Pod 状態確認
kubectl get pods -n openbao -l app.kubernetes.io/name=openbao -o wide
```

```bash
# 各 Pod の sealed / unsealed 状態
for i in 0 1 2; do
  echo "=== openbao-${i} ==="
  kubectl exec -n openbao openbao-${i} -- bao status 2>/dev/null || echo "Pod not responsive"
done
```

```bash
# Raft クラスタの状態
kubectl exec -n openbao openbao-0 -- bao operator raft list-peers
```

```bash
# tier1 facade の Secret 取得エラーを確認
kubectl logs -n k1s0 deploy/tier1-facade --tail=50 | grep -iE "secret|openbao|vault"
```

ステークホルダー通知（即時）:

- SEV1 即時宣言、Slack `#incident-sev1` に「OpenBao no leader、tier1 全 API 影響」を投稿。
- [`oncall/escalation.md`](../../oncall/escalation.md) を起動、CTO + Security SRE に連絡。
- Status Page を「主要機能停止」に更新。

## 4. 原因特定手順

```bash
# 各 Pod のログ確認
for i in 0 1 2; do
  echo "=== openbao-${i} ==="
  kubectl logs -n openbao openbao-${i} --tail=50
done

# OpenBao Operator ログ
kubectl logs -n openbao deploy/openbao-injector --tail=100

# Raft state ファイルの破損チェック
kubectl exec -n openbao openbao-0 -- ls -la /vault/data/raft
```

よくある原因:

1. **Raft 過半数喪失（2 ノード以上同時障害）**: Pod / Node 障害で過半数が応答不能。生存ノードが leader 選出を継続的に試みるが quorum に達しない。
2. **ディスク full**: `/vault/data` が満杯で Raft log の追加不可。`du -sh /vault/data` で確認。
3. **TLS 証明書期限切れ**: cluster_address 通信用の TLS 証明書が失効。[`RB-SEC-002`](RB-SEC-002-cert-expiry.md) と連鎖。
4. **設定ミスによる sealed 状態**: Pod 再起動後に auto-unseal が失敗（KMS / Cloud Key 接続エラー）。手動 unseal が必要。
5. **Network Partition**: Pod 間通信ができず leader 選出失敗。Cilium / Calico ログ確認。

エスカレーション: Raft state ファイル破損が確認された場合は L3 起案者 + 外部コミュニティ（OpenBao GitHub）に支援依頼。

## 5. 復旧手順

### Case A: 1 Pod のみ障害（quorum 維持）

自動 failover で別 Pod が leader 昇格を待つ:

```bash
kubectl get pods -n openbao -w
# 障害 Pod を強制削除
kubectl delete pod openbao-1 -n openbao
```

### Case B: Sealed 状態（Pod 再起動後）

各 Pod を unseal する（Shamir 5/3 構成、3 share 必要）:

```bash
# 各 Pod に対して 3 回 unseal を実行
for i in 0 1 2; do
  echo "=== unseal openbao-${i} ==="
  for share in <SHARE_1> <SHARE_2> <SHARE_3>; do
    kubectl exec -n openbao openbao-${i} -- bao operator unseal "${share}"
  done
done

# 状態確認
for i in 0 1 2; do
  kubectl exec -n openbao openbao-${i} -- bao status
done
```

### Case C: Raft 過半数喪失（2 Pod 障害）

最後に生存していた Pod から手動で peer を削除し、recovery mode で起動:

```bash
# 障害 peer を削除
kubectl exec -n openbao openbao-0 -- bao operator raft remove-peer openbao-1
kubectl exec -n openbao openbao-0 -- bao operator raft remove-peer openbao-2

# 単一ノード mode で leader 復活確認
kubectl exec -n openbao openbao-0 -- bao status
```

その後、新 Pod を joiner として再投入:

```bash
kubectl delete pod openbao-1 openbao-2 -n openbao
# StatefulSet が再作成、自動的に raft join を試みる

# join が走らない場合は手動 join
kubectl exec -n openbao openbao-1 -- \
  bao operator raft join \
  -leader-ca-cert=@/vault/userconfig/openbao-tls/ca.crt \
  https://openbao-0.openbao-internal:8200
```

### Case D: Raft state 完全破損

最終手段として snapshot からリストア:

```bash
# 最新 snapshot を取得（事前に定期取得が必要）
kubectl exec -n openbao openbao-0 -- bao operator raft snapshot save /tmp/snap.snap

# 全 Pod 削除 + PVC クリア + snapshot リストア
kubectl scale statefulset openbao -n openbao --replicas=0
kubectl delete pvc -n openbao -l app.kubernetes.io/name=openbao
kubectl scale statefulset openbao -n openbao --replicas=3
# Pod 起動後に snapshot リストア
kubectl exec -n openbao openbao-0 -- bao operator raft snapshot restore /tmp/snap.snap
```

### tier1 facade の接続復旧

OpenBao が unsealed + leader 復活したら:

```bash
kubectl rollout restart deployment/tier1-facade -n k1s0
kubectl rollout status deployment/tier1-facade -n k1s0
```

## 6. 検証手順

復旧完了の判定基準:

- 全 3 Pod が `Running` かつ `Ready=True`。
- 全 Pod で `bao status` が `Sealed: false` かつ `Initialized: true`。
- `bao operator raft list-peers` で 3 voters が `voter` 役で表示。
- `openbao_core_unsealed{namespace="openbao"} == 1` が 5 分間継続。
- `openbao_raft_leader_lease_seconds > 0`（leader が選出されている）。
- tier1 facade の `/healthz` が 200、`tier1_secret_fetch_errors_total` の rate が 0 に収束。
- 直近 10 分の Loki クエリ `{namespace="k1s0"} |= "openbao" |= "ERROR"` が 0 件。
- `bao read sys/health` が `200 OK` を返す。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`postmortems/<YYYY-MM-DD>-RB-SEC-001.md`）。
- Raft snapshot の定期取得（CronJob、6h 間隔）が稼働しているか確認。未稼働なら整備（`infra/security/openbao/cluster/snapshot-cron.yaml`）。
- Auto-unseal（Cloud KMS）の設定確認 — 採用後の運用拡大時で導入予定だがリリース時点では Shamir 手動。
- PodDisruptionBudget で同時 evict を 1 Pod に制限（`spec.maxUnavailable: 1`）。
- TLS 証明書の有効期限を Grafana ダッシュボードに表示（[`RB-SEC-002`](RB-SEC-002-cert-expiry.md) 連動）。
- NFR-A-REC-002 の MTTR ログを更新（目標: 暫定 30 分以内、恒久 2 時間以内）。
- 月次 Chaos Drill 対象に「OpenBao 1 Pod kill」シナリオを追加（`ops/chaos/experiments/pod-delete/openbao.yaml`）。
- 四半期に 1 回、Shamir share 保有者の連絡先確認と模擬 unseal 演習。

## 8. 関連 Runbook

- 関連設計書: `infra/security/openbao/values.yaml`、[`docs/05_実装/85_Identity設計/secrets-matrix.md`](../../../docs/05_実装/85_Identity設計/secrets-matrix.md)
- 関連 ADR: [ADR-SEC-002（OpenBao）](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md)
- 関連 NFR: [NFR-E-AC-001 / NFR-E-AC-004](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md)
- 関連 FMEA: [FMEA-002](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — TLS 証明書期限切れが原因の場合
  - [`../secret-rotation.md`](../secret-rotation.md) §「OpenBao unseal share」 — Shamir 再分散
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md) — DB 接続情報を OpenBao が保持しているため連鎖影響
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
