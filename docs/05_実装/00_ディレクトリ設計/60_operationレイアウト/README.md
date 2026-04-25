# 60. operation レイアウト

本章は `deploy/` と `ops/` の配置を確定する。ADR-DIR-002 で規定した 3 階層分離（infra / deploy / ops）の下 2 層、すなわち GitOps 配信定義と運用領域を担う。

## 本章の対象

- `deploy/`: GitOps 配信定義（ArgoCD Application / ApplicationSet / Helm chart / Kustomize overlay / Argo Rollouts / OpenTofu / Image Updater）
- `ops/`: 運用領域（Runbook / Chaos / DR / Oncall / Load test / Scripts）
- `tools/backstage/` 相当のプラグイン配置

infra/（素構成）と明確に分離される。配信定義の書換えはアプリコードを触らずに可能であり、逆に運用 Runbook は実行状況の記録と手順管理に集中する。

## 本章の構成

| ファイル | 内容 |
|---|---|
| 01_deploy配置_GitOps.md | `deploy/` 全体のサブディレクトリ配置 |
| 02_ArgoCD_ApplicationSet配置.md | `deploy/apps/` の ArgoCD Application / ApplicationSet |
| 03_Helm_charts配置.md | `deploy/charts/` の共通 Helm chart |
| 04_Kustomize_overlays配置.md | `deploy/kustomize/` の base + overlays |
| 05_ops配置_Runbook_Chaos_DR.md | `ops/` 全体と各サブディレクトリ |
| 06_Backstage_プラグイン配置.md | Backstage プラグインの配置先 |
| 07_OpenTofu配置.md | `deploy/opentofu/` のベアメタルプロビジョン |

## 本章で採番する IMP-DIR ID

- IMP-DIR-OPS-091（deploy 配置 GitOps）— `01_deploy配置_GitOps.md`
- IMP-DIR-OPS-092（ArgoCD ApplicationSet 配置）— `02_ArgoCD_ApplicationSet配置.md`
- IMP-DIR-OPS-093（Helm charts 配置）— `03_Helm_charts配置.md`
- IMP-DIR-OPS-094（Kustomize overlays 配置）— `04_Kustomize_overlays配置.md`
- IMP-DIR-OPS-095（ops 配置 Runbook/Chaos/DR）— `05_ops配置_Runbook_Chaos_DR.md`
- IMP-DIR-OPS-096（Backstage プラグイン配置）— `06_Backstage_プラグイン配置.md`
- IMP-DIR-OPS-097（OpenTofu 配置）— `07_OpenTofu配置.md`

予約 IMP-DIR ID は `IMP-DIR-OPS-098` 〜 `IMP-DIR-OPS-110`（運用蓄積後で採番）。本章採番範囲は `IMP-DIR-OPS-091` 〜 `IMP-DIR-OPS-110`。

## 対応 ADR

- ADR-DIR-002（infra / deploy / ops 3 階層分離）
- ADR-CICD-001（GitOps）
- ADR-CICD-002（ArgoCD 採用）
- ADR-CICD-003（Argo Rollouts 採用）
