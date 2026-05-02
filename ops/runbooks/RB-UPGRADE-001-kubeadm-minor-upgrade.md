---
runbook_id: RB-UPGRADE-001
title: kubeadm minor version upgrade（N → N+1）
category: OPS
severity: SEV2
owner: 起案者
automation: manual
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 N/A / 恒久 30〜45 分/pair
last_updated: 2026-05-02
---

# RB-UPGRADE-001: kubeadm minor version upgrade（N → N+1）

本 Runbook は ADR-TEST-005 の Upgrade drill 月次実施 + production K8s minor version upgrade 直前に staging で必ず実走する手順を定める。ADR-INFRA-001 の kubeadm 採用と完全整合し、kubeadm 公式 plan/apply/node 経路を逸脱しない。

詳細手順は採用初期で完成させる。本リリース時点では skeleton として 8 セクション枠 + 主要 step を整備する。

## 1. 前提条件

- 実行者は staging cluster の cluster-admin 権限を持つ
- kubeadm / kubectl のバージョンが現行 K8s version と整合（pkgs.k8s.io 公式 repo）
- staging cluster が control-plane 3 + worker 3 の HA 構成（ADR-INFRA-001）
- L4 標準テスト（後続再構築予定）が前回 PASS（baseline 確認）

## 2. 対象事象

- 月次 Upgrade drill schedule（cron）が起動した、または production upgrade 直前
- K8s upstream の minor release 公開（年 3 回、`kubeadm upgrade plan` で表示）

## 3. 初動手順（5 分以内）

```bash
# 現行 version 確認
kubectl version
kubeadm version
# upgrade 候補表示
kubeadm upgrade plan
```

ステークホルダー通知: Slack `#status` に「staging upgrade 開始 N → N+1」を投稿。

## 4. 原因特定手順

upgrade 失敗時の原因分類:

1. **deprecated API 残存**: `kubectl get --show-managed-fields ...` で対象を特定
2. **CRD 互換崩壊**: Operator が新 K8s API に未対応
3. **kubelet バージョン不整合**: worker の `apt-mark hold` 確認

## 5. 復旧手順

```bash
# 1 control-plane で upgrade apply
sudo kubeadm upgrade apply v1.<N+1>.0

# 2/3 control-plane で upgrade node
sudo kubeadm upgrade node

# 各 worker で drain → upgrade → uncordon
kubectl drain <worker> --ignore-daemonsets --delete-emptydir-data
# worker の VM 上で:
sudo apt-mark unhold kubelet kubeadm && sudo apt-get update && sudo apt-get install -y kubelet kubeadm && sudo apt-mark hold kubelet kubeadm
sudo kubeadm upgrade node
sudo systemctl daemon-reload && sudo systemctl restart kubelet
kubectl uncordon <worker>
```

## 6. 検証手順

```bash
# 全 node が target version で Ready
kubectl get nodes
# tier1 namespace の Pod が継続稼働
kubectl get pods -n tier1-state
# L4 標準テスト PASS（テスト基盤再構築後に手順反映）
```

## 7. 予防策

- staging で月次 drill を必ず先行（production 前提）
- 失敗時の Runbook 修正 commit を tests/.upgrade-results.md に記録

## 8. 関連 Runbook

- [RB-DR-001 etcd-snapshot-restore](incidents/RB-DR-001-etcd-snapshot-restore.md) — upgrade 失敗時の roll back
- ADR-TEST-005 / ADR-INFRA-001（L4 verify は後続テスト基盤再構築後に追記）
