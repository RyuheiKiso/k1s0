---
id: plan.overview.scenario_ops_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.target_use_case
  - plan.development_team_structure
  - arch.ops.ops_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D, E]
  proof_classes: []
---

# ops 担当者シナリオ INDEX

## 一文方針

ops 担当者（シニア級、メタ専任）が 5 signal_class × 5 phase ops loop を軸に踏む 14 シナリオを 1 ファイル 1 シナリオで列挙する。on-call rotation / postmortem / runbook 整備 / progressive delivery 立会 / SRE 50% rule 維持 / break-glass 実行が本軸の核心であり、blameless culture と fatigue budget の遵守をすべてのシナリオに通底させる。

## 担当者プロフィール

| 属性 | 内容 |
|------|------|
| 級 | シニア（メタ専任） |
| 必須スキル | SRE 50% rule / SLO / MWMBR alerting / Argo Rollouts / Backstage TechDocs / Tekton |
| 責務 | 5 signal_class × 5 phase ops loop / on-call rotation / postmortem / runbook |
| 想定人数 | 3-5 名 |
| 主要デバイス | ラップトップ（Mattermost / GitHub / Backstage / Prometheus / Argo CD 常時参照） |
| 認証手段 | パスワード + WebAuthn（FIDO2 セキュリティキー） |

詳細は [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md) を参照。

## 5 signal_class 定義

| signal_class | 説明 |
|---|---|
| latency | 処理遅延（P95 / P99 レイテンシ） |
| throughput | スループット（RPS / msg/s） |
| error | エラー率（5xx / DLQ dead-letter 率） |
| saturation | リソース飽和度（CPU / memory / queue depth） |
| availability | 可用性（稼働率 / SLO error budget 残余） |

## 5 phase ops loop 定義

| phase | 説明 |
|---|---|
| detect | Mattermost / pager alert 受信 → signal_class 特定 |
| triage | 影響範囲・severity 確定 → escalation 先判断 |
| contain | 障害影響範囲の局所化（traffic shed / rate limit 等） |
| recover | サービス正常化 → SLO 復帰確認 |
| improve | postmortem → runbook 更新 → toil 削減 backlog 登録 |

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | [on_call_rotation_handover](01_on_call_rotation_handover.md) | on-call rotation shift 切替時 (8h 毎) | 日次 (3 回) | 運用ループ適合仕様 / SLO 適合仕様 | [周期] |
| 02 | [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md) | SLO burn rate alert 受信時 | 週次〜月次 | SLO 適合仕様 / 運用ループ適合仕様 | [緊急] |
| 03 | [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md) | incident 6 phase の最終 phase 到達時 | 週次〜月次 | 運用ループ適合仕様 / security 強制機構 | [周期] |
| 04 | [runbook整備_Backstage_TechDocs](04_runbook整備_Backstage_TechDocs.md) | 新規 incident class 発生時 / 月次 toil 見直し時 | 月次 | 運用ループ適合仕様 / 検証規律適合仕様 | [周期] |
| 05 | [Argo_Rollouts_progressive_delivery](05_Argo_Rollouts_progressive_delivery.md) | 新 release の progressive delivery 開始時 | 週次〜月次 | SLO 適合仕様 / クラスタ位相適合仕様 | [周期] |
| 06 | [ops_edge_cluster運用](06_ops_edge_cluster運用.md) | 月次定期確認 / target cluster 障害発生時 | 月次 + イベント駆動 | ops_edge_cluster / 運用ループ適合仕様 | [周期]+[緊急] |
| 07 | [break_glass_execution](07_break_glass_execution.md) | 本番 DB cluster-admin が緊急必要になった時 | 年次〜不定期 | 認証適合仕様 / security 強制機構 | [緊急] |
| 08 | [SRE_50pc_rule_toil削減](08_SRE_50pc_rule_toil削減.md) | 月次 toil review 日程到来時 | 月次 | 運用ループ適合仕様 | [周期] |
| 09 | [incident_response_7class主導](09_incident_response_7class主導.md) | incident が検知された時 | 週次〜月次 | 運用ループ適合仕様 / security 強制機構 | [緊急] |
| 10 | [observability_5signal_dashboard整備](10_observability_5signal_dashboard整備.md) | 新軸 SLO 追加時 / SLO 閾値変更時 / 月次 review 時 | 月次 | 観測適合仕様 / 運用ループ適合仕様 | [周期] |
| 11 | [capacity_planning_KEDA調整](11_capacity_planning_KEDA調整.md) | 月次 capacity review 日程到来時 | 月次 | テナント容量適合仕様 / クラスタ位相適合仕様 | [周期] |
| 12 | [chaos_drill_観察_signal評価](12_chaos_drill_観察_signal評価.md) | chaos drill 実施日程到来時 | 月次 | 運用ループ適合仕様 / 検証規律適合仕様 | [周期] |
| 13 | [release_切り_dual_sign_off](13_release_切り_dual_sign_off.md) | release milestone 達成時 | 月次〜四半期 | 運用ループ適合仕様 / 検証規律適合仕様 | [計画] |
| 14 | [24h_oncall_fatigue_budget管理](14_24h_oncall_fatigue_budget管理.md) | 月次 fatigue budget review 日程到来時 | 月次 | 運用ループ適合仕様 | [周期] |

## ops 担当者の決定権限境界

| 操作 | ops 担当者の権限 | escalation 先 |
|------|----------------|--------------|
| SLO burn rate alert への初動対応 | 実施可 | — |
| on-call rotation 調整 | 実施可 | — |
| runbook 更新（Backstage TechDocs） | 実施可 | — |
| Argo Rollouts rollback 承認 | 実施可 | — |
| **break-glass (v1_emergency_step_up) 起票・実行** | **security 担当者との dual sign-off 必須** | security 担当者 |
| **schema migration を伴う recover 操作** | **実施不可** | data 担当者 |
| **KEK rotate / crypto-shred** | **実施不可** | data 担当者 + security 担当者 |
| **新規 Kyverno admission policy 追加** | **実施不可** | infra 担当者 |

## 新規参画者向けオンボーディング

- **Day 1-3**: README 全体読了 → 01（on-call handover）→ 02（SLO burn rate alert）の一文方針と流れを通読
- **Day 4-7**: 09（incident response 7 class 主導）→ 03（postmortem PR merge gate）で incident 対応フローを演習
- **Week 2**: 04（runbook 整備）→ 08（SRE 50% rule toil 削減）で月次 review を担当
- **Week 3-4**: 05（Argo Rollouts）→ 13（release sign-off）で progressive delivery と release フローを把握
- **Month 2 以降**: 07（break-glass）→ 06（ops-edge cluster）は発生時・月次に担当。11（capacity planning）→ 12（chaos drill）で定期 review を継続

## シナリオ間の依存関係

```
02 (SLO burn rate alert)
  └─► 09 (incident response 主導) — burn rate 超過が incident class 宣言に昇格

09 (incident response)
  └─► 03 (postmortem PR merge gate) — incident 終結後に postmortem PR を作成

03 (postmortem PR merge gate)
  └─► 13 (release 切り dual sign-off) — postmortem PR merge が release の物理 prerequisite

04 (runbook 整備)
  └─► 08 (SRE 50% rule toil 削減) — runbook 整備による toil 削減効果を 08 の backlog に登録

11 (capacity planning)
  └─► 05 (Argo Rollouts progressive delivery) — capacity 設定更新後に次 release の progressive delivery 設定に反映

12 (chaos drill 観察)
  └─► 04 (runbook 整備) — drill 結果 postmortem が runbook 更新のトリガになる

14 (fatigue budget 管理)
  └─► 01 (on-call handover) — rotation 調整結果が handover 構成に反映
```

## 関連参照

- [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)
- [ターゲットと利用シナリオ index](../README.md)
- `arch.ops.ops_index`
