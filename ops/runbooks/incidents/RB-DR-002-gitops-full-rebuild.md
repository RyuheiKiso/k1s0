---
runbook_id: RB-DR-002
title: GitOps 完全再構築（経路 B、RTO 4 時間）
category: DR
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 N/A / 恒久 4 時間
last_updated: 2026-05-02
---

# RB-DR-002: GitOps 完全再構築（DR drill 経路 B）

ADR-TEST-005 経路 B。snapshot 不在で cluster 全壊した時の最終手段。tofu apply + kubeadm init + Argo CD 同期で Git 上の全 manifest から復元。RTO 4 時間。詳細は採用後の運用拡大時で完成、本リリース時点は skeleton。

## 1. 前提条件

- IaC（OpenTofu）で VM 構築 manifest が Git 管理されている
- Argo CD app-of-apps が Git に最新 commit 済
- 手作業 `kubectl apply` を禁じた SoT 規律（ADR-POL-002）が production で守られている

## 2. 対象事象

- 経路 A（etcd snapshot 復旧）失敗
- snapshot 自体が破損 / 紛失
- VM 全壊（disk 物理破壊）

## 3. 初動手順（5 分以内）

Slack `#status` に「経路 B 起動、RTO 4 時間」を宣言。経路 A 失敗の根本原因を別調査として並走起動。

## 4. 原因特定手順

経路 B 起動時点で「経路 A が機能しない」が確定。以後は復旧に集中、原因調査は別 incident として別 ticket 化。

## 5. 復旧手順

```bash
# Step 1: VM 再構築（tofu apply、約 30 分）
cd infra/environments/production
tofu apply

# Step 2: kubeadm init で空 cluster 構築（約 20 分）
ssh cp-1 'sudo kubeadm init --config /etc/kubeadm-init.yaml'
# join command を取得して残 2 control-plane + worker を join

# Step 3: Argo CD インストール（約 10 分）
kubectl apply -f deploy/apps/app-of-apps/

# Step 4: Argo CD sync（全 manifest を Git から復元、約 2 時間）
argocd app sync --all
argocd app wait --all --timeout 7200

# Step 5: PostgreSQL barman-cloud restore（経路 C を呼ぶ）
# RB-DR-003 を参照
```

## 6. 検証手順

```bash
kubectl get nodes
kubectl get applications -n argocd | head -50
# 全 namespace が想定の Pod 数で Running
make verify-e2e
```

## 7. 予防策

- 手作業 `kubectl apply` 禁止規律（ADR-POL-002）の厳格運用
- Argo CD app-of-apps を毎 commit で Git 同期
- IaC manifest の dry-run を月次 staging で実施

## 8. 関連 Runbook

- [RB-DR-001 etcd snapshot restore](RB-DR-001-etcd-snapshot-restore.md) — 経路 A
- [RB-DR-003 postgres barman restore](RB-DR-003-postgres-barman-restore.md) — Step 5 で呼ぶ
- ADR-TEST-005 / ADR-POL-002 / ADR-CICD-001
