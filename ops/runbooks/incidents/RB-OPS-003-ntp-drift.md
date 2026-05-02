---
runbook_id: RB-OPS-003
title: 時刻ずれ（NTP drift）対応
category: OPS
severity: SEV3
owner: 協力者
automation: manual
alertmanager_rule: NodeTimeDrift
fmea_id: 間接対応
estimated_recovery: 暫定 15 分 / 恒久 2 時間
last_updated: 2026-05-02
---

# RB-OPS-003: 時刻ずれ（NTP drift）対応

本 Runbook は Kubernetes Node の NTP 時刻ずれが 100ms を超えた時の対応を定める。
Loki / Tempo / Audit ログのタイムライン相関が壊れ、インシデント対応が困難になる SEV3 リスク。
NFR-C-NOP-004 / DS-OPS-RB-013 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + Node 上の SSH 権限（debug Pod 経由）。
- 必要ツール: `kubectl` / `chronyc` / `timedatectl`。
- kubectl context が `k1s0-prod`。
- chrony が DaemonSet として全 Node に配置されていること。

## 2. 対象事象

- Alertmanager `NodeTimeDrift` 発火（`abs(node_timex_offset_seconds) > 0.1` を 5 分継続）、または
- Loki / Tempo の trace ID 相関で時刻不整合が観測される。

検知シグナル:

```promql
# Node 時刻オフセット
abs(node_timex_offset_seconds) > 0.1

# chrony が同期している NTP server
node_timex_sync_status == 0
```

通知経路: Slack `#alert-infra` → SRE 当番。

## 3. 初動手順（5 分以内）

```bash
# 影響 Node の特定
kubectl get nodes
kubectl exec -n monitoring -l app=node-exporter --field-selector spec.nodeName=<NODE> -- \
  cat /proc/timex | grep -E "offset|status"
```

```bash
# debug Pod で chrony 状態確認
kubectl debug node/<NODE> -it --image=ghcr.io/nicolaka/netshoot -- \
  chronyc tracking
```

## 4. 原因特定手順

よくある原因:

1. **NTP サーバ到達性**: 上位 NTP サーバ（`ntp.example.jp` / `pool.ntp.org`）に届かない。
2. **chrony デーモン停止**: chrony Pod がクラッシュ。
3. **VM ホスト時刻ずれ**: Cloud Provider の VM ホスト自体がずれている（rare）。
4. **仮想化レイヤの時刻同期**: VMware / Hyper-V のホスト同期設定が無効。

## 5. 復旧手順

### chrony 強制再同期

```bash
kubectl debug node/<NODE> -it --image=ghcr.io/nicolaka/netshoot -- \
  chronyc burst 4/4
kubectl debug node/<NODE> -it --image=ghcr.io/nicolaka/netshoot -- \
  chronyc makestep
```

### chrony Pod 再起動

```bash
kubectl rollout restart daemonset/chrony -n kube-system
```

### NTP サーバ切替

`infra/k8s/multinode/chrony-config.yaml` を編集して上位 NTP server を切替。

## 6. 検証手順

- `node_timex_offset_seconds < 0.05` が 5 分継続。
- `chronyc tracking` で `Stratum < 5` かつ `Last offset < 100ms`。
- Loki / Tempo の trace ID 相関が正常動作（手動でクロスリファレンス）。

## 7. 予防策

- ポストモーテム起票（1 週間以内、`postmortems/<YYYY-MM-DD>-RB-OPS-003.md`）。
- NTP サーバ到達性の監視追加（`network_blackbox_probe`）。
- chrony 設定の Hardening（`makestep 1.0 3` 等）。

## 8. 関連 Runbook

- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §DS-OPS-RB-013
- 関連 NFR: [NFR-C-NOP-004](../../../docs/03_要件定義/30_非機能要件/C_運用.md)
- 連鎖 Runbook: [`RB-SEC-002-cert-expiry.md`](RB-SEC-002-cert-expiry.md) — ClockSkew 起因の証明書検証失敗
