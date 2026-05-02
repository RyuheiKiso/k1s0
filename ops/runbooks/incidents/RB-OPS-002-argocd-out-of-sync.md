---
runbook_id: RB-OPS-002
title: Argo CD Out-of-Sync 長期化対応
category: OPS
severity: SEV3
owner: 協力者
automation: manual
alertmanager_rule: ArgocdOutOfSyncProlonged
fmea_id: 間接対応
estimated_recovery: 暫定 30 分 / 恒久 2 時間
last_updated: 2026-05-02
---

# RB-OPS-002: Argo CD Out-of-Sync 長期化対応

本 Runbook は Argo CD Application が 30 分以上 `Out-of-Sync` 状態が継続した時の対応を定める。
GitOps の一貫性が崩れ、本番状態が Git と乖離する SEV3 リスク。NFR-C-OPS-001 に対応する。

## 1. 前提条件

- 実行者は Argo CD admin 権限。
- 必要ツール: `argocd` / `kubectl`。
- kubectl context が `k1s0-prod`。

## 2. 対象事象

- Alertmanager `ArgocdOutOfSyncProlonged` 発火（`argocd_app_info{sync_status!="Synced"}` を 30 分継続）、または
- Argo CD ダッシュボードで複数 app が `OutOfSync`。

検知シグナル:

```promql
# Out-of-Sync 状態の app 数
sum(argocd_app_info{sync_status="OutOfSync"}) > 0

# 同期遅延（最終 reconciliation からの経過時間）
time() - argocd_app_reconcile_count > 1800
```

通知経路: Slack `#dev-ops` → 該当 app オーナーに周知。

## 3. 初動手順（5 分以内）

```bash
# Out-of-Sync app の一覧
argocd app list | grep -v Synced

# 該当 app の詳細
argocd app get <app-name>
argocd app diff <app-name>
```

## 4. 原因特定手順

よくある原因:

1. **Manifest 競合**: 手動で `kubectl edit` した変更が Git に反映されていない。
2. **RBAC 不足**: Argo CD ServiceAccount が namespace の resource を変更できない。
3. **リソース不足**: Pod が Pending（Node 容量不足）。
4. **CRD 未インストール**: 新規追加リソースの CRD が未デプロイ。
5. **Webhook 失敗**: Validating Admission Webhook が拒否。

## 5. 復旧手順

### 手動 Sync 試行

```bash
argocd app sync <app-name>
argocd app wait <app-name> --health --timeout 300
```

### 競合解消

```bash
# 手動変更を Git に反映
argocd app diff <app-name> > /tmp/diff.txt
# diff 内容を Git に commit + PR
```

### CRD 不足の場合

```bash
# CRD を先にデプロイ
kubectl apply -f infra/<crd-path>/
argocd app sync <app-name>
```

### Hard refresh

```bash
argocd app sync <app-name> --replace --force --prune
```

## 6. 検証手順

- 該当 app が `Synced Healthy`。
- `argocd app diff` で差分なし。
- Manifest 競合の場合、Git 反映済み。

## 7. 予防策

- ポストモーテム起票（1 週間以内、`postmortems/<YYYY-MM-DD>-RB-OPS-002.md`）。
- 手動変更禁止のポリシー徹底（Kyverno で `argocd.argoproj.io/managed-by` ラベル必須）。
- Argo CD の auto-sync を全 app で有効化。

## 8. 関連 Runbook

- 関連 ADR: [ADR-CICD-001 Argo CD](../../../docs/02_構想設計/adr/ADR-CICD-001-argocd.md)
- 連鎖 Runbook: [`RB-OPS-001-cicd-pipeline-down.md`](RB-OPS-001-cicd-pipeline-down.md)
