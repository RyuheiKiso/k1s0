---
id: plan.overview.scenario_ops_argo_rollouts_progressive_delivery
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.ops.ops_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# Argo_Rollouts_progressive_delivery

## 一文方針

Argo Rollouts による canary / blue-green progressive delivery の立会で、analysis template の 5 signal 評価と自動 rollback 条件を確認し、release を安全に進める。

> 水曜午後 2 時、ops 担当者が Argo Rollouts dashboard を開く。警報配信コンポーネント v1.8.0 の canary rollout が開始しており、10% traffic で analysis template が 5 signal を評価中だ。latency P99 が baseline より 8% 増加している。自動 rollback 閾値は 15% だが、ops 担当者は手動での rollback を判断するか、次の 25% ステップへ進むかを注視する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: Argo Rollouts の progressive delivery を立会し、5 signal 評価と rollback 条件を確認して release を安全に完了させる

## 現状業務での痛み

- 新 release のデプロイを一括 rollout するため、問題発生時の影響範囲が全 traffic になる
- rollback 判断が属人的で、どのメトリクス値で rollback すべきか基準が明文化されていない
- canary / blue-green の traffic 切替を手動操作するため、担当者不在時に rollout が止まる
- 5 signal の評価結果が分散し、rollout 判断に必要な情報を集める時間がかかる

## k1s0 でこう変わる

- Argo Rollouts の analysis template が 5 signal（latency / throughput / error / saturation / availability）を自動評価し、閾値超過で自動 rollback が走る
- ops 担当者は立会・承認・手動介入の役割に集中でき、一括 rollout のリスクがなくなる
- rollback 条件が analysis template に明文化され、属人的判断を排除する
- canary → 25% → 50% → 100% の段階的 traffic 増加が自動化され、担当者不在でも rollout が継続する

## Trigger

新 release の progressive delivery 開始時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 週次〜月次
- 典型きっかけ: 「`release_gate.lock.yaml` 全 cell green を確認し、警報配信コンポーネント v1.8.0 の canary rollout を開始した」
- 頻度根拠: release 頻度に依存。月次〜週次の release cadence を想定

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Argo Rollouts dashboard / Prometheus | 5 signal 評価立会・rollback 判断・rollout 承認 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub PR list | rollback 時の hotfix 対応（escalation 受け） |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Kubernetes dashboard | cluster resource 確認（escalation 受け） |

## 個人 KPI / 達成感

- progressive delivery 中の自動 rollback 発生率: 目標 10% 以下（リリース品質向上指標）
- canary 開始から 100% traffic 到達までの所要時間: SLA 以内
- rollout 中の SLO error budget 消費: baseline 比 5% 以内

## 工数 / 関与人数 / コスト感

- 立会時間（1 release）: 2〜4 時間（canary → 100% まで）
- rollback 発生時の追加工数: 1〜2 時間（原因確認 + hotfix 対応依頼）
- 関与人数: 1〜3 名（ops + 必要に応じて tier1 / infra）

## 前提

- Argo Rollouts が cluster に導入され、analysis template が 5 signal を評価するよう設定されている
- `release_gate.lock.yaml` 全 cell green が rollout 開始の物理 prerequisite として機能している
- Prometheus / Mimir に analysis template が参照する SLO metric が存在する

## 流れ

1. **rollout 開始確認**: `release_gate.lock.yaml` 全 cell green を確認後、Argo Rollouts で canary rollout を開始する
2. **canary 10% traffic 評価**: analysis template が 5 signal（latency / throughput / error / saturation / availability）を評価する。評価結果を Argo Rollouts dashboard で確認する
3. **手動 rollback 判断**: latency P99 が baseline 比 15% 超、または error rate が 0.5% 超の場合は手動 rollback を実行する。自動 rollback 閾値（20%）に達しない場合でも ops 担当者の裁量で rollback 可能
4. **25% → 50% → 100% 段階承認**: 各ステップで analysis template の評価が pass している場合、次ステップへの承認を Argo Rollouts dashboard で実行する
5. **100% traffic 到達確認**: 全 traffic が new version に切り替わり、SLO dashboard で正常稼働を確認する
6. **rollout 完了報告**: Mattermost `#ops-releases` に rollout 完了を報告し、`release_gate.lock.yaml` の状態を更新する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | ops 担当者 | release_gate 全 cell green 確認・canary 開始 | Mattermost: 「[rollout] v1.8.0 canary 10% 開始」 |
| T+10m | Argo Rollouts | analysis template 評価開始（5 signal） | — |
| T+20m | ops 担当者 | 10% traffic 評価結果確認 | — |
| T+30m | ops 担当者 | 25% 承認（評価 pass の場合） | — |
| T+60m | ops 担当者 | 50% 承認 | — |
| T+90m | ops 担当者 | 100% 承認 | — |
| T+100m | Argo Rollouts | 100% traffic 切替完了 | Mattermost: 「[rollout] v1.8.0 100% traffic 切替完了」 |
| T+110m | ops 担当者 | SLO dashboard 最終確認・報告 | Mattermost: 「[rollout] v1.8.0 rollout 完了。SLO 正常」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信コンポーネントの rollout は availability SLO に直結 |
| FA 生産指示 | 高 | 生産指示配信の rollout 中 latency SLO 維持 |
| ライン稼働監視 | 中 | ライン稼働監視コンポーネントの rollout 立会 |

## 関連適合仕様 / 関連 OSS

- SLO 適合仕様
- クラスタ位相適合仕様
- 関連 OSS: Argo Rollouts（progressive delivery）/ Prometheus / Mimir（SLO metric）/ Mattermost（通知）

## 期待結果 / 観測指標

- canary rollout が analysis template の 5 signal 評価を全 step で pass している
- 100% traffic 到達後に SLO error budget 消費が baseline 比 5% 以内
- rollback が発生した場合、原因が特定されホットフィックスが tier1 に escalation されている

## 失敗時の挙動 / escalation

- **analysis template が自動 rollback を実行した**: Mattermost `#ops-releases` に自動通知。ops 担当者が 5 signal の評価結果を確認し、root cause を特定して tier1 に hotfix を escalation する。SLA: rollback 発生から 2 時間以内に原因特定
- **analysis template の評価が stuck する（10 分以上 pending）**: infra 担当者に Prometheus / Mimir の metric 取得エラーを確認するよう escalation する
- **手動 rollback 実行後に old version でも SLO 違反が続く**: ops + infra + tier1 で emergency incident として対応する

## 失敗パターン (anti-pattern)

1. **`release_gate.lock.yaml` の全 cell green を確認せずに rollout を開始する**: CVE 未対応や postmortem 未完了のまま release が走るリスクがある
2. **analysis template の評価結果を確認せず全 step を自動で進める**: 問題の早期発見が遅れ、100% traffic で初めて障害が顕在化する
3. **rollback 閾値を超えても「様子見」で次ステップに進む**: SLO error budget が急速に消費され、error budget 枯渇で後続 release がすべてブロックされる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [release_切り_dual_sign_off](13_release_切り_dual_sign_off.md)
- [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md)
