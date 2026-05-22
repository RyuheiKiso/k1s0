---
id: detail.formal.ops_dx
axis: formal
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.formal.formal_index
  - arch.formal.proof_artifact_policy
  - arch.formal.counter_example_policy
  - arch.formal.proof_review_policy
  - detail.formal.formal_conformance
  - detail.formal.formal_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
trace:
  fr_ids:
  - FR-formal-003

---

# formal 運用 UI と開発者体験

## 一文方針
- formal の運用 UI は Perses dashboard（proof matrix / counter-example list / reviewer queue / SLO 4 種）と ChatOps bot（PR review 割当 / break-glass / audit trail）の 2 経路で提供、proof artifact 編集は IDE plugin（VS Code / IntelliJ）+ Tekton Pipeline preview で支援、proof tool の実行は CI runner と long-running night batch の双方を提供する。

## Perses dashboard 構成

### proof matrix dashboard（19 axis × 5 proof_class）
- 各 cell が現在 status（verified / in_progress / failed / accepted_with_assumption）を色 coding
- cell click で `proof_inventory` entry に drill-down、obligation 詳細 + tool 出力 + reviewer sigs 表示
- cadence breach 予兆（last_verified_at が cadence_days × 0.8 を超えた cell）を warning 色で alert

### counter-example list dashboard
- open entries の severity / close_due_at / axis_id / obligation_id を一覧
- severity=high の open は別 widget で強調
- close_due_at までの remaining days を sort 可能

### reviewer queue dashboard
- 自分に割当てられた pending review、待機 PR、各 PR の deadline
- reviewer 別 review velocity（per quarter）

### SLO 4 種 dashboard
- `v1_proof_coverage_compliance`: 19 × 5 cell の verified / accepted_with_assumption 比率
- `v1_proof_freshness_compliance`: cadence 内 cell 比率
- `v1_counter_example_closure_compliance`: close_due_at 越え cell ゼロ比率
- `v1_proof_review_dual_signoff_compliance`: dual signoff 完備比率

## ChatOps bot 機能

### `/formal review-assign <PR>`
- 該当 PR の proof artifact 変更に基づき reviewer を round-robin 割当
- cooldown / domain expertise を考慮
- reviewer に notification + dashboard link

### `/formal break-glass <obligation_id> <reason> <ticket>`
- OpenBao response wrapping で短期 cosign signing token を発行
- `audit_event` subject に `v1_formal_break_glass` fact emit
- 24 hour 以内に retro review 完了を要求

### `/formal status <obligation_id>`
- 該当 obligation の current status / last_verified_at / counter-examples / reviewers を表示

### `/formal cycle <axis_id>`
- 軸全体の proof cycle（specify → model → prove → review → archive）の各 phase 状況を表示

### `/formal counter-example-triage <id>`
- counter-example の severity 判定 + close_kind 候補を提示

## IDE plugin

### VS Code plugin
- TLA+ / P / Stainless / Dafny / Lean / Kani の syntax highlight + LSP（official extensions の単一深耕、商用 fork は採用しない）
- proof obligation の inline preview（軸 spec の `formal_obligations` 段落から自動表示）
- counter-example trace の inline visualization
- reviewer queue の inline 表示

### IntelliJ plugin
- Stainless（Scala）/ Lean（経由 Lean4 IntelliJ）の official plugin
- proof obligation の inline preview

### vim / emacs
- 各 tool の official syntax / LSP support のみ、独自 plugin は持たない（単一深耕）

## proof tool 実行環境

### CI runner（軽量 proof）
- per PR で起動、SMT timeout 30 min/module
- Tekton Pipeline で並列起動、Argo Workflows で orchestration
- tool 出力 log を ClickHouse SoR に emit

### long-running night batch（重い proof）
- Apalache の large state space sweep、Lean の重い tactic、Kani の bound parameter sweep
- night batch 専用 cluster（infra01 spot capacity）
- 結果は morning に dashboard 反映 + reviewer notification

### on-demand proof（developer が IDE から trigger）
- IDE plugin から Tekton Pipeline 経由で proof 実行、結果を IDE に return
- timeout 5 min/proof（重い proof は night batch に投げる）

## developer onboarding

### tutorial
- TLA+ + Apalache: 1 hour（counter examples を見つけた経験を持つ）
- P: 1 hour
- Stainless: 2 hour（Scala 経験前提）
- Dafny: 2 hour
- Lean 4 + mathlib: 4 hour（理論研究員向け）
- Kani: 1 hour（Rust 経験前提）
- CBMC: 1 hour（C / C++ 経験前提）

### cookbook
- 軸別の proof obligation 定型 pattern
- safety / liveness / refinement / program correctness / runtime modelcheck の代表例
- counter-example triage の標準手順

### office hours
- weekly で proof office hours、domain expert が proof 設計の相談を受ける

## migration drill UI
- tool major version migration drill 起動 UI:
    - `tla_apalache_pin` / `mathlib_pin` / `kani_cbmc_pin` の new version を candidate 登録 → drill 起動 → 全 proof 再 verify → diff report → dual review で commit
- migration drill の cadence:
    - tool major version: 6 month に 1 回
    - mathlib upstream pull: monthly
    - bound parameter review: quarterly

## break-glass UX
- emergency 時の cosign signing token 発行 UI:
    - ChatOps bot から `/formal break-glass` コマンド
    - actor の cosign keypair で sign + ticket_ref 必須
    - 発行 log を `audit_event` + Perses break-glass dashboard に表示
    - 24 hour timer で retro review 完了状況を track

## 採用しない設計
- GUI 中心の proof editor（Why3 IDE 等）: 採用しない、LSP + IDE plugin の標準 OSS 経路のみ
- 商用 proof IDE: 採用しない（[非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md) 参照）
- 文章 only の proof catalog（README.md に列挙のみ）: 禁止、必ず machine-readable lock.yaml + dashboard
- dashboard を multiple platform（Perses + Grafana 並列運用）: 採用しない、Perses 単一深耕（09 観測と整合）
- ChatOps を Slack 専用: 禁止、ChatOps bot は IRC / Slack / Mattermost / Teams 等の API agnostic（ops 13 と整合）

## audit / 不可逆性
- 全 operator action（break-glass / approval / override）は `audit_event` subject に `v1_formal_<action>` fact emit
- ChatOps bot は dual reviewer signoff を要する operation を `cosign signed approval` で物理 enforce

## 関連参照
- [formal 設計方針 index](../../03_概要設計/11_formal設計方針/README.md)
- [形式検証適合仕様](../01_適合仕様/20_形式検証適合仕様.md)
- [formal 強制機構](../02_強制機構/10_formal強制機構.md)
- [proof_review 方針](../../03_概要設計/11_formal設計方針/08_proof_review方針.md)
