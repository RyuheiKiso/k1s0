---
id: plan.overview.drawio_figure_index
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.target_use_case
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# drawio 図一覧 index

## 一文方針

`docs/01_企画/05_ターゲットと利用シナリオ/` 配下に追加する drawio 図の計画一覧。図作成は `drawio-authoring` skill 規約（白背景・矢印重なり禁止・1 図 1 ページ・コントラスト下限）および `figure-layer-convention` skill 規約（アプリ層 / ネットワーク層 / インフラ層 / データ層の 3 軸分離）に従う。

## 作図規約リンク

- drawio-authoring skill: `.claude/skills/drawio-authoring/SKILL.md`
- figure-layer-convention skill: `.claude/skills/figure-layer-convention/SKILL.md`
- ファイル形式: `.drawio` + `.svg` を同一コミットで管理
- 埋込形式: `![図の説明](img/xxx.svg)` (alt 必須、空 alt 禁止)

## 計画図一覧

### 横断図 (05/ 直下 `img/` に配置予定)

| # | ファイル名 | 説明 | 関連ドキュメント | 状態 |
|---|---|---|---|---|
| 1 | `img/persona_map_11.drawio` | 11 ペルソナ relationship map (業務担当者→tier3→tier2→tier1→infra/data/security の業務フロー) | 00_ペルソナ詳細カード集.md | 未作成 |
| 2 | `img/cross_axis_dependency_graph.drawio` | 11 軸間の依存 graph (軸ノード 11 個、依存矢印 30 本) | 00_cross_axis依存マトリクス.md | 未作成 |
| 3 | `img/24h_timeline_handover.drawio` | 24h 時間軸 × 11 担当者 swimlane + on-call handover 3 回 | 00_24h時間軸俯瞰_handover.md | 未作成 |
| 4 | `img/industry_9_business_x_persona_heatmap.drawio` | 9 業務 × 11 軸 影響度 heatmap (4 段階) | 00_業界9業務カバレッジマトリクス.md | 未作成 |

### 軸別図 (各軸 `img/` 配下に配置予定)

| # | ファイル名 | 説明 | 関連シナリオ | 状態 |
|---|---|---|---|---|
| 5 | `03_tier3担当者シナリオ/img/client_state_4_layer_fsm.drawio` | 4 layer client state (ST/OL/PQ/DR) 遷移図 + 25h オフライン復帰フロー | tier3-04 オフライン業務記録 | 未作成 |
| 6 | `03_tier3担当者シナリオ/img/business_conflict_4_subtype_branch.drawio` | BusinessConflict 4 subtype (stale_write/lost_update/supersede/concurrent_edit) の UI 分岐 decision tree | tier3-02 業務エラー UX | 未作成 |
| 7 | `01_tier1担当者シナリオ/img/bidi_transport_two_layer.drawio` | bidi 二層化 (Application semantic layer / Transport adapter layer) + HTTP/2 / HTTP/3 / WebTransport 各経路 | tier1-07 Transport 切替 | 未作成 |
| 8 | `05_data担当者シナリオ/img/dr_failover_sequence_5_actors.drawio` | DR cross-region failover の T+0 〜 T+5 分の 5 actor 間 sequence 図 | data-14 DR cross-region failover | 未作成 |
| 9 | `04_infra担当者シナリオ/img/topology_failover_5_class_timeline.drawio` | 5 topology_class × failover orchestrator + drill cadence Timeline | infra-02 topology drill failover | 未作成 |
| 10 | `06_security担当者シナリオ/img/kek_shamir_ceremony_architecture.drawio` | OpenBao Transit / HSM / shamir M-of-N / cosign witness chain の architecture flow | security-06 KEK shamir ceremony | 未作成 |
| 11 | `06_security担当者シナリオ/img/audit_4_layer_immutability.drawio` | 4 層 immutability (WORM Object Lock / hash chain / RFC 3161 / Sigstore Rekor) architecture | security-09 audit hash chain | 未作成 |
| 12 | `06_security担当者シナリオ/img/cve_critical_6_phase_timeline.drawio` | 7 incident class × 6 phase playbook の Timeline 図 | security-07 CVE CRITICAL incident | 未作成 |
| 13 | `07_業務担当者シナリオ/img/business_operator_24h_journey.drawio` | 業務担当者の 1 日 Journey map (出社→朝礼→ライン→検査→承認→帰宅) | 07-01 出社 / 02 検査 / 11 帰宅 | 未作成 |
| 14 | `07_業務担当者シナリオ/img/offline_25h_state.drawio` | 25h オフライン業務記録 state machine (Server Truth/OL/PQ/DR 遷移 + TTL 超過) | 07-03 オフライン業務記録 | 未作成 |
| 15 | `08_業務管理者シナリオ/img/business_admin_state.drawio` | Backstage プラグイン操作の state machine (主要操作フロー) | 08 軸全般 | 未作成 |
| 16 | `09_外部監査人シナリオ/img/external_auditor_verification_workflow.drawio` | 外部監査人の audit hash chain 検証 workflow (独立検証 5 ステップ) | 09-01〜05 | 未作成 |
| 17 | `10_ops担当者シナリオ/img/ops_oncall_3layer_rotation.drawio` | on-call 3 階層 rotation (8h × 3 = 24h cycle + escalation path) | 10-01 handover / 10-09 incident | 未作成 |

## 作成手順 (各図共通)

1. `.claude/skills/drawio-authoring/templates/canvas.drawio` を雛形に作成
2. 2 レイヤ以上登場する場合は `figure-layer-convention` の 4 レイヤ色 + 凡例ブロック必須
3. `bin/drawio-lint` ERROR ゼロ → `bin/drawio-export` で SVG エクスポート → `bin/svg-postcheck` ERROR ゼロ
4. SVG を md に `![alt](img/xxx.svg)` で埋込 (alt 必須・空 alt 禁止)
5. `.drawio` と `.svg` を同一コミットで commit

## 関連参照

- [ターゲットと利用シナリオ index](README.md)
- drawio-authoring skill（外部リソース）
