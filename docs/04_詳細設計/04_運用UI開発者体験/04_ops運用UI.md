---
id: detail.ops.ops_dx
axis: ops
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.ops.ops_index
  - arch.ops.alert_policy
  - arch.ops.oncall_policy
  - arch.ops.runbook_policy
  - arch.ops.toil_reduction_policy
  - detail.ops.ops_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# ops 運用 UI と開発者体験

## 一文方針
- 運用 UI は Backstage（infra 13 / data 13 / security 13 と共有）+ Perses（SLO / burn rate / toil / change calendar / postmortem dashboard）+ 自製 escalation engine（Argo Workflows + Mattermost）+ Mattermost（incident channel / ChatOps）+ 内部 custom plugin（ops_loop coverage / runbook map / toil tracker）の 5 OSS 経由のみで提供し、開発者は IDE 拡張 + pre-commit hook + Testcontainers で同等の ops check を local で完結できる。

## 運用 UI（operator 視点）

### Backstage TechDocs
- runbook_catalog の visualization（class × executable のヒートマップ）
- alert_catalog の visualization（alert ⇔ runbook 1:1 coverage）
- ownership_table の visualization（service ↔ team ↔ rotation 三項関係）
- postmortem archive の search / index
- on-call training material（blameless culture / incident command roles / break-glass protocol）

### Perses dashboard
- SLO burn rate（service 別 / SLO 別、MWMBR の fast / slow / medium 並列表示）
- error budget 残量 / consumption history
- active incident（incident.io OSS / GitHub Issue query）
- active freeze（service / namespace 別の freeze status）
- change calendar（次 7 日の planned change / freeze window）
- toil_minutes_per_week（個人 / team / 軸別、quarterly target との比較）
- on-call paging budget（個人 / team 別、fatigue budget violation flag）
- postmortem PR status（open / draft / review / merged の phase 別 count + SLA 違反 flag）
- automation backlog（class / priority / status 別 count）
- canary status（active rollout の AnalysisTemplate metric 並列表示）

### 自製 escalation engine（Argo Workflows + Mattermost）
- rotation visualization（過去 / 現在 / 未来の shift grid）
- escalation history（page → ack → resolve の timeline）
- active incident page log
- rotation gap warning（roster.lock.yaml と現在 state の drift）

### Mattermost incident channels
- `#ops-incident-<service>`: active incident の incident channel、IC + CL + SME + scribe による live coordination
- `#ops-incident-confidential`: PII / security adjacency / legal sensitive incident、限定 membership
- `#ops-status-page`: customer impact のある incident の internal status mirror
- `#ops-toil`: toil event 通知 + automation backlog discussion
- `#ops-change-management`: freeze 通知 / change calendar update

### 内部 custom plugin（Backstage）
- `ops_loop_coverage_visualizer`: 5 phase × 軸 × signal_class の grid に物理 action 経路を highlight、未 cover cell を warn
- `runbook_map_navigator`: alert → runbook → linked incident → linked postmortem → linked action item の chain 双方向 navigate
- `toil_tracker`: 個人 / team の `toil_minutes_per_week` をリアルタイム表示、SRE 50% rule violation warning + automation backlog の P0/P1 link
- `canary_observability`: active rollout の SLO metric を canary stage 別に時系列表示

## 開発者 UI（developer 視点）

### IDE 拡張
- VSCode 拡張（自社 / OSS、v1 では README + 設定 guide のみ提供、v2 で自社拡張検討）:
    - PR diff 上で change_classes 注釈の自動 suggest
    - alert_catalog の touched alert に対応する runbook を inline 表示
    - SLO budget 影響の visualize（PR がどの SLO を touch するか hint）
    - feature flag lifecycle warning（v1_release_flag が 90 日越え cleanup overdue 警告）

### pre-commit hook（git pre-commit、自社 boilerplate 提供）
- alert_catalog の change で runbook 1:1 coverage check
- `rollouts/<service>/analysis-template.yaml` の generator output 一致 check
- `flags.lock.yaml` の flag class + cleanup_due_at 注釈必須 check
- change_calendar の global freeze window 内 PR の v1_emergency 注釈 check
- postmortem PR の 6 必須 section header 一致 check
- blameless lint（Semgrep custom rule for postmortem）

### PR 上の自動 comment
- PR の change_class 注釈に基づく risk summary（rollout strategy / freeze gate status / SLO impact prediction）
- ownership_table が touched された場合の rotation impact summary
- touched alert の runbook coverage delta
- touched flag の cleanup deadline summary
- active incident / postmortem pending warning

### Testcontainers ベース local
- 全 ops Kyverno admission policy（freeze / rollout / cleanup overdue 等）を local cluster（k3d / kind）で同一 build artifact から適用
- flagd dev mode で feature flag toggle を local に再現
- Tekton Pipelines を local 起動して runbook の dry-run 実行を可能化
- Argo Rollouts を local cluster で再現、canary AnalysisTemplate の dev test
- Litmus light scenario で local chaos drill 再現

## 自動化の規律
- 全 dashboard / TechDocs / Perses panel は build artifact（`dashboards/` 配下 yaml）から生成、UI 上の手書き編集は drift として CI 検出 + Argo CD self-heal
- runbook_catalog / alert_catalog / ownership_table / rotation_roster の編集 PR は CODEOWNERS で関連 axis owner（service team / ops lead / SRE lead）必須 review
- automation_backlog の priority 変更 PR は ops lead + SRE lead の二人 reviewer 必須

## アクセシビリティ
- 全 dashboard の色指定は drawio 規約 / figure-layer-convention と整合（colorblind-safe palette）
- UI 操作の全認証は OIDC + WebAuthn 必須、password-only 経路ゼロ
- Mattermost slash command による ChatOps は keyboard only でも完結、screen reader 互換

## 開発者向け onboarding 導線
- 新規 service team が ops 規律を踏まえて onboard する手順を Backstage TechDocs に集約:
    1. service の `ownership_table.lock.yaml` entry 追加 PR（4 軸 ownership）
    2. service の SLO 宣言（tier1/13 と双方向 lock）
    3. SLO に対応する alert_catalog entry 追加
    4. alert に対応する runbook 作成（段 1 + 段 2）
    5. AnalysisTemplate generator の input 設定
    6. canary delivery strategy の宣言
    7. shadow rotation 加入 + on-call training
- 上記 1〜7 すべての完了が production deploy の前提（未完了 service の deploy は Kyverno で物理 block）

## 採用しない方針
- 商用 dashboard SaaS（DataDog / New Relic / Honeycomb）: 採用しない
- UI 上の policy / rotation / runbook 直接編集: 禁止
- 個人 development 環境への production dashboard 同期: 禁止
- runbook_catalog / alert_catalog の手書き: 禁止
- dashboard の personalization: 禁止
- 開発者の独自 alert routing 持ち込み: 禁止
- ChatOps の audit log 不在経路: 禁止

## 関連参照
- [ops 設計方針 index](../../03_概要設計/08_ops設計方針/README.md)
- [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md)
- [ops 強制機構](../02_強制機構/07_ops強制機構.md)
- [ops_edge_cluster](../03_クロスカッティング適合仕様/10_ops_edge_cluster.md)
