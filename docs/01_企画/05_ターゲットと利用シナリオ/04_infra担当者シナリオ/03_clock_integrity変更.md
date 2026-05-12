---
id: plan.infra.scenario_clock_integrity_change
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# clock integrity 変更

## 一文方針

データセンター / クラウドプロバイダの PTP 対応変更や leap second 戦略見直しに際して、[5 clock_integrity_class](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md) すべてで HLC skew allowance と leap second smear 設定の整合性を保ちながら安全に変更を適用する。

## Trigger（発火条件）

- データセンター / クラウドプロバイダの PTP 対応変更が通知された時
- leap second 戦略の見直しが必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（DC 変更時 / クラウドプロバイダ PTP 対応変更時、年間 0-2 件）。典型きっかけ: 「新データセンターへの移行で PTP Grand Master が変わり、全 node の chronyd 設定を更新する必要が生じた」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: dual reviewer

## 前提

- 5 clock_integrity_class（PTP / NTP / HLC / leap second 戦略）が `clock_integrity.lock.yaml` に記録済みであること

## 流れ

1. 変更対象の clock_integrity_class を特定（例: v1_zone_replicated → PTP 精度要件の変更）
2. PTP（IEEE 1588）or NTP tier の設定を OpenTofu IaC で更新
3. k8s node の chronyd / ptpd 設定を Kustomize overlay で変更
4. HLC（Hybrid Logical Clock）の skew allowance が新設定で維持されることを integration test で確認
5. leap second 処理: smear 設定が全 node で統一されていることを lint で確認（混在は禁止）
6. 変更後に `clock_integrity.lock.yaml` を更新し dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- 時刻整合適合仕様: [../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)
- 関連 OSS: OpenTofu / Kustomize / chronyd / ptpd

## 期待結果 / 観測指標

- 変更後も HLC skew allowance が仕様値以内に収まること
- 全 node で leap second smear 設定が統一されていること
- `clock_integrity.lock.yaml` が新設定で更新されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- HLC skew allowance 超過 → 変更 rollback + 原因究明 + PR 差し戻し
- leap second smear 混在検出 → lint fail で apply 中止 + 全 node 設定の統一修正
- integration test fail → `clock_integrity.lock.yaml` 更新せず + **postmortem 期限: 2 営業日以内**（Backstage runbook `clock-integrity-postmortem`）

**escalate 先**: ops 担当者 / Mattermost `#infra-incident`
**SLA: 30 分以内（拘束）**。超過した場合は ops 担当者に Mattermost `#infra-incident` で即時報告
**runbook**: Backstage runbook `clock-integrity-rollback` を参照

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針（clock integrity 変更の方針・背景を確認するため）
