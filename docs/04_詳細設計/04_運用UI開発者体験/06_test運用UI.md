---
id: detail.test.ops_dx
axis: test
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.test.test_index
  - arch.test.coverage_matrix_policy
  - arch.test.regression_policy
  - detail.test.test_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
trace:
  fr_ids:
  - FR-test-003

---

# test 運用 UI と開発者体験

## 一文方針
- test 軸の運用 UI と開発者体験は、Backstage TechDocs + Perses dashboard + GitHub PR + Mattermost ChatOps の既存 UI 道具立てに subordinate し、新規 UI を test 軸が独自開発することは原則しない。test 軸が要求する UX は (a) coverage matrix の Perses dashboard 化、(b) PR 上の machine-readable check、(c) Backstage 上の test ownership UI、(d) ChatOps slash command の 4 経路のみで完結する。

## 設計原則
- **既存 UI 上での投影**: Backstage / Perses / GitHub PR / Mattermost に投影することで test 軸の独自 UI 開発を禁止
- **機械可読性 first**: developer 向け表示は全て test artifact から導出される build artifact、手書き dashboard yaml は cap 5%
- **local replay**: developer は local で coverage cell の test を replay できる
- **audit trail**: ChatOps slash command（test 軸関連）は全件 audit_event subject に `v1_test_chatops` fact emit

## Backstage TechDocs 上の test カード
- 各 service の Backstage entity に test card を表示:
    - coverage matrix 中の本 service 関連 cell の green / red 状態
    - mutation_score current / quarter monotonic 推移
    - flaky_quarantine 中の test 一覧
    - pact verify can-i-deploy 状態
    - chaos drill last_green_at + 次回 cadence
    - test ownership 4 軸（service / domain / framework / drill）
- 全 card は coverage_matrix.lock.yaml / mutation_score.lock.yaml 等から導出
- 手動編集禁止、Argo CD self-heal で drift detect

## Perses dashboard

### dashboard 1: coverage matrix overview
- panel: 18 axis × 5 verification_class の grid を heatmap として表示、green / red / quarantined / pending の色分け
- panel: last_green_at の cadence 越え cell の一覧

### dashboard 2: mutation_score quarter trend
- panel: module_id ごとの quarter score 推移、monotonic increase の物理可視化
- panel: equivalent mutant 認定数の推移

### dashboard 3: flaky quarantine
- panel: quarantine 中 test 一覧、quarantine_due_at までの残日数
- panel: quarantine 期限超過の警告

### dashboard 4: performance baseline
- panel: 各 perf_scenario_id の SLI baseline quarter 推移、regression 検出の物理可視化

### dashboard 5: drill cadence
- panel: v1_fault_chaos / v1_scenario_replay の last_green_at cadence 表、cadence 越え cell の一覧

- 全 dashboard は build artifact（`dashboards/test/*.json`）として管理

## GitHub PR check

### PR check（必須 green）
- `test_coverage_matrix_check`: coverage_matrix の green 維持
- `test_mutation_score_check`: mutation_score monotonic 維持
- `test_flaky_quarantine_check`: flaky_quarantine の cap 越えなし
- `test_pact_can_i_deploy_check`: Pact Broker の can-i-deploy green
- `test_chaos_drill_cadence_check`: chaos blueprint の cadence 越えなし
- `test_regression_corpus_added_check`: defect 修正 PR に regression_corpus_added 注釈
- `test_snapshot_signed_check`: snapshot 更新 PR が drill_owner ack 済み

- PR check 失敗時は merge block（Kyverno admission policy + GitHub branch protection の double）
- PR 上の表示は GitHub Action の job summary に build artifact から生成された markdown を投稿

## Mattermost ChatOps slash command

| command | 内容 |
|---|---|
| `/test replay <scenario_id>` | shadow cluster 上で v1_scenario_replay を起動、Tekton Pipeline trigger |
| `/test mutate <module_id>` | mutation testing を on-demand 起動、quarter baseline 比較 |
| `/test chaos dryrun <blueprint_id>` | chaos blueprint の dry-run（v1_shadow_only のみ）|
| `/test pact verify <pair_id>` | Pact verify を on-demand 起動 |
| `/test flaky list` | quarantine 中 test 一覧表示 |
| `/test cell <axis> <class>` | 特定 cell の状態 + last_green_at + ownership 表示 |

- 全 ChatOps 操作は audit_event subject に `v1_test_chatops` fact emit、必須 column: actor / scenario_or_blueprint_id / ack_pointer / blast_radius（chaos の場合）/ ticket_ref

## 開発者 UI（developer 視点）

### IDE 拡張
- VSCode 拡張（v1 では README + 設定 guide のみ提供、v2 で自社拡張検討）:
    - PR diff 上で coverage 影響の自動 suggest
    - flaky_quarantine 中の test inline 表示
    - mutation_score 影響の visualize

### pre-commit hook（git pre-commit、自社 boilerplate 提供）
- coverage_matrix が touched された場合の Conftest check
- properties.yaml / pacts.yaml / scenarios.yaml の generator output drift check
- snapshot mask の cap 20% 以下 check

### PR 上の自動 comment
- PR の changed 軸 + changed verification_class cell の状態
- mutation_score の quarter delta
- flaky_quarantine の cap 越え警告

### Testcontainers ベース local
- 全 test admission policy（coverage cell green / mutation monotonic 等）を local cluster（k3d / kind）で同一 build artifact から適用
- Tekton Pipelines を local 起動して runbook の dry-run 実行
- Litmus light scenario で local chaos drill 再現

## 自動化の規律
- 全 dashboard / TechDocs / Perses panel は build artifact から生成、UI 上の手書き編集は drift として CI 検出 + Argo CD self-heal
- coverage_matrix / mutation_modules / chaos_blueprints の編集 PR は CODEOWNERS で関連 axis owner（service team / drill_owner）必須 review

## 採用しない方針
- 商用 dashboard SaaS（DataDog / New Relic）: 採用しない
- UI 上の coverage matrix 直接編集: 禁止、PR 経由のみ
- 個人 development 環境への production dashboard 同期: 禁止
- coverage_matrix / mutation_score の手書き: 禁止
- dashboard の personalization: 禁止
- 開発者の独自 test artifact 持ち込み: 禁止

## audit / 不可逆性
- 全 operator action（snapshot update / equivalent mutant decision / corpus write）は audit_event subject に `v1_test_<action>` fact emit
- ChatOps bot は dual reviewer signoff を要する operation を `cosign signed approval` で物理 enforce

## 関連参照
- [test 設計方針 index](../../03_概要設計/10_test設計方針/README.md)
- [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md)
- [test 強制機構](../02_強制機構/09_test強制機構.md)
