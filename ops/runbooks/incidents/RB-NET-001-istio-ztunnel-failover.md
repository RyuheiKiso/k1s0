---
runbook_id: RB-NET-001
title: Istio ztunnel 障害対応
category: NET
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: IstioZtunnelDown
fmea_id: FMEA-005
estimated_recovery: 暫定 10 分 / 恒久 2 時間
last_updated: 2026-05-02
---

# RB-NET-001: Istio ztunnel 障害対応

本 Runbook は Istio Ambient のノードローカル ztunnel が停止した時の対応を定める。
影響は該当 Node 上の Pod 間 mTLS 通信のみ（他 Node の Pod は影響なし）であり SEV2。
NFR-E-AC-002 / NFR-A-CONT-001 / FMEA-005 / ADR-0003（Istio Ambient）に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + `istio-system` namespace の DaemonSet 編集権限。
- 必要ツール: `kubectl` / `istioctl`。
- kubectl context が `k1s0-prod`。
- Istio Ambient の ztunnel が DaemonSet として全 Node に配置されていること（`infra/mesh/istio-ambient/`）。

## 2. 対象事象

- Alertmanager `IstioZtunnelDown` 発火（`up{namespace="istio-system",job="ztunnel"} == 0` を 60 秒継続）、または
- 該当 Node 上の Pod 間通信が `503` ハンドシェイク失敗、または
- アプリケーションの分散トレースで `connection reset by peer` が頻発。

検知シグナル:

```promql
# ztunnel Pod の稼働状態
up{namespace="istio-system", job="ztunnel"} == 0

# Istio Ambient Pod の TLS ハンドシェイク失敗率
sum(rate(istio_requests_total{response_code="503", source_workload_namespace="k1s0-tier1"}[5m])) > 0.05

# ztunnel メモリ消費（OOM 兆候）
container_memory_working_set_bytes{pod=~"ztunnel-.*"} / container_spec_memory_limit_bytes > 0.9
```

ダッシュボード: **Grafana → k1s0 Istio Ambient**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#alert-mesh`。

## 3. 初動手順（5 分以内）

```bash
# ztunnel DaemonSet の状態
kubectl get daemonset ztunnel -n istio-system -o wide
kubectl get pods -n istio-system -l app=ztunnel -o wide
```

```bash
# 障害ノードと Pod の特定
kubectl get pods -n istio-system -l app=ztunnel \
  --field-selector status.phase!=Running -o wide
```

```bash
# 障害 Pod のログ
NODE=$(kubectl get pod <ztunnel-pod> -n istio-system -o jsonpath='{.spec.nodeName}')
kubectl logs -n istio-system <ztunnel-pod> --tail=100
kubectl describe pod <ztunnel-pod> -n istio-system
```

```bash
# 該当 Node 上の影響 Pod
kubectl get pods --all-namespaces -o wide --field-selector spec.nodeName=${NODE} \
  | grep -v Running
```

ステークホルダー通知: Slack `#alert-mesh` に「Istio ztunnel 障害、Node ${NODE} 上の Pod 間通信影響」を投稿。
全 Node の ztunnel が同時障害なら SEV1 昇格して `oncall/escalation.md` 起動。

## 4. 原因特定手順

```bash
# Istio Control Plane のヘルス
istioctl proxy-status

# ztunnel の eBPF プログラム状態
kubectl exec -n istio-system <ztunnel-pod> -- \
  bpftool prog show

# Node 上の iptables / network namespace
kubectl debug node/${NODE} -it --image=ghcr.io/nicolaka/netshoot -- \
  iptables -t nat -L
```

よくある原因:

1. **ztunnel Pod OOMKilled**: メモリ Limit に対して接続数が多い。`kubectl top pod -n istio-system` で確認。
2. **eBPF プログラムロード失敗**: Linux カーネルバージョンが要件（>=5.10）を満たさない、または kernel module 未ロード。
3. **iptables ルール破損**: Node 上で外部ツールが iptables を変更。
4. **TLS 証明書（SVID）失効**: SPIRE からの SVID 取得失敗。[`RB-SEC-002`](RB-SEC-002-cert-expiry.md) と連鎖。
5. **Istio Control Plane（istiod）障害**: configuration push が止まる。`istioctl proxy-status` で確認。

エスカレーション: eBPF / kernel 起因の場合は L3 起案者 + Linux カーネル知識者へ連絡。

## 5. 復旧手順

### Pod 再起動（〜2 分）

```bash
kubectl delete pod <ztunnel-pod> -n istio-system
# DaemonSet が即時再作成
kubectl get pods -n istio-system -l app=ztunnel -o wide -w
```

### Node 上の iptables / network namespace 確認

```bash
kubectl debug node/${NODE} -it --image=ghcr.io/nicolaka/netshoot -- \
  ip netns list

# iptables を再生成（istio が再起動時に正しいルールを書き直す）
kubectl rollout restart daemonset/ztunnel -n istio-system
```

### Node 再起動（最終手段）

```bash
# Node を drain して上の Pod を退避
kubectl cordon ${NODE}
kubectl drain ${NODE} --ignore-daemonsets --delete-emptydir-data

# Cloud Provider で Node を再起動（GCP の場合）
gcloud compute instances reset ${NODE} --zone=<zone>

# Node が Ready に戻ったら uncordon
kubectl uncordon ${NODE}
```

### 該当 Node 上の Pod の通信回復確認

```bash
kubectl get pods -A -o wide --field-selector spec.nodeName=${NODE}
# tier1-facade を rolling restart して新接続を確立
kubectl rollout restart deployment/tier1-facade -n k1s0
```

## 6. 検証手順

復旧完了の判定基準:

- 全 Node で `up{namespace="istio-system",job="ztunnel"} == 1` が 5 分継続。
- 該当 Node 上の Pod 間通信エラー（`istio_requests_total{response_code="503"}`）が 1% 未満。
- `istioctl proxy-status` が全 Pod について `SYNCED`。
- 該当 Node 上の Pod に対する `kubectl exec ... -- curl https://<service>` が成功。
- 直近 10 分の Loki クエリ `{namespace="k1s0"} |= "503" |= "ztunnel"` が 0 件。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-NET-001.md`）。
- ztunnel メモリ Limit を見直し（`infra/mesh/istio-ambient/`）。
- Linux カーネル version の固定（GKE / EKS の Node Image Pin）。
- 月次 Chaos Drill 対象に「ztunnel Pod kill」シナリオを追加。
- NFR-A-REC-002 の MTTR ログを更新。

## 8. 関連 Runbook

- 関連設計書: `infra/mesh/istio-ambient/`
- 関連 ADR: [ADR-0003 Istio Ambient](../../../docs/02_構想設計/adr/ADR-0003-istio-ambient.md)
- 関連 NFR: [NFR-E-AC-002 / NFR-A-CONT-001](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 FMEA: [FMEA-005](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — SPIRE SVID 期限切れが原因の場合
  - [`RB-NET-002-envoy-dos.md`](RB-NET-002-envoy-dos.md)（予定） — Gateway 層障害が同時発生した場合
