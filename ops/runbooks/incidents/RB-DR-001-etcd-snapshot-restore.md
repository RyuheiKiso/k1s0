---
runbook_id: RB-DR-001
title: etcd snapshot からの復旧（経路 A、RTO 30 分）
category: DR
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 30 分 / 恒久 30 分
last_updated: 2026-05-02
---

# RB-DR-001: etcd snapshot からの復旧（DR drill 経路 A）

ADR-TEST-005 の DR drill 4 経路ローテーション中の経路 A。`etcdctl snapshot restore` で control-plane 状態を復元し、RTO 30 分以内で cluster を機能再開する。詳細は採用後の運用拡大時で完成、本リリース時点は skeleton。

## 1. 前提条件

- 直近 24h 以内の etcd snapshot が MinIO に保管されている
- 全 control-plane の SSH アクセス権 + sudo
- etcdctl / kubeadm が control-plane VM に install 済

## 2. 対象事象

- etcd 全 3 ノード Quorum 喪失（kube-apiserver 503）
- 検知: PromQL `up{job="etcd"} == 0` 全 3 instance、または `kubectl get nodes` が timeout

## 3. 初動手順（5 分以内）

```bash
# control-plane 1 台で etcd 状態確認
sudo crictl ps | grep etcd
sudo systemctl status kubelet
```

Slack `#status` に「etcd 全壊検知、経路 A 復旧開始」を投稿。SEV1 escalation 起動。

## 4. 原因特定手順

- ストレージ破損（disk full / fsck エラー）
- 人為的 `etcdctl del --prefix=""`
- 災害（VM disk 破壊）

## 5. 復旧手順

```bash
# control-plane 1 台で復旧（残 2 台は kubeadm reset で初期化）
mc cp minio/k1s0-backup/etcd/snapshot-<latest>.db /var/lib/etcd-restore/
sudo etcdctl snapshot restore /var/lib/etcd-restore/snapshot-<latest>.db \
  --data-dir /var/lib/etcd-new
sudo mv /var/lib/etcd /var/lib/etcd-broken
sudo mv /var/lib/etcd-new /var/lib/etcd
sudo systemctl restart kubelet
# 残 2 台が control-plane 復帰: kubeadm join
```

## 6. 検証手順

```bash
kubectl get nodes
kubectl get pods -A | head -50
# L4 standard E2E が PASS
make verify-e2e
```

## 7. 予防策

- 日次 etcd snapshot CronJob の動作確認
- 四半期 DR drill 経路 A の継続実施
- staging cluster で同手順を月次 dry-run

## 8. 関連 Runbook

- [RB-DR-002 GitOps full-rebuild](RB-DR-002-gitops-full-rebuild.md) — snapshot 不在時の経路 B
- ADR-TEST-005 / 02_etcd全ノード障害.md
