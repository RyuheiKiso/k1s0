---
id: detail.cross_schema.apicurio_gitops_sot
axis: cross_schema
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - detail.tier1.schema_evolution_conformance
  - detail.tier1.oss_lifecycle_conformance
  - detail.data.preservation_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - tier1
  - data
  - client
  - infra
---

# Apicurio Registry GitOps SoT 適合仕様（v1）

## 位置づけ
- Apicurio Registry の subject schema / per-subject compatibility rule / global rule を **git SoT** で管理し、Argo CD + Apicurio Operator が各 cluster の Apicurio instance に declarative reconcile する経路を一箇所に固定
- 09_data レプリケーション方針 / 14_データ保全適合仕様 / 06_スキーマ進化適合仕様 / client 12_コード生成方針 / infra GitOps 配信方針 / security 強制機構 / 検証規律適合仕様 / 08_OSS ライフサイクル適合仕様 と双方向 lock
- Apicurio Registry の `/rules` API metadata（compatibility rule / global rule）が MirrorMaker2 で同期できない物理制約に対する代替経路として、GitOps SoT で cross-cluster equality を eventual から strong に格上げ

## 設計原則
- 単一の真は git。Apicurio Registry instance は「git lock の物理 cache」として再定義する。Registry の state（subject schema / rule）は git diff = empty を CI gate で物理 enforce
- Registry API への直接 write は禁止。`block-direct-apicurio-write` admission policy が物理 enforce
- Apicurio Operator service account のみが Registry の write API を呼べる。他の service account / 人間 user は read-only
- 06_スキーマ進化適合仕様 の「単一の真は『位置』ではなく『導出関係』で守る」原則を Apicurio に適用。git SoT → Operator reconcile → Registry instance → Library codegen → 業務コード という導出関係の一方向性を CI で物理 enforce
- L1+ 単一深耕（apicurio_registry primary）と L2\*（karapace pair）の twin 規律を維持。Karapace へ demote する場合も同 governance を維持し、Registry instance のみ差替え

## git repo layout
- SoT repo は `apicurio_subjects/` path（GitOps monorepo の一部、infra GitOps 配信方針 と整合）
- layout:
  - `global_rules/`
    - `compatibility.yaml`: 全 subject default
    - `validity.yaml`: 全 subject 妥当性 check
    - `integrity.yaml`: reference 整合
  - `groups/<group_id>/artifacts/<artifact_id>/`
    - `metadata.yaml`: artifact 種別、labels、所有
    - `schema/v1.avsc` / `v2.avsc` / ...
    - `rules/compatibility.yaml` / `validity.yaml`
  - `apicurio_rules.lock.yaml`: build artifact
- schema 本体（`*.avsc` 等）の content-addressed hash を artifact metadata に bundle。manifest 形式（CUE schema）で type 検証

## `apicurio_rules.lock.yaml`（build artifact）
- 全 subject / rule / schema version の static 自己宣言を GitOps build script が集約して生成する build artifact。手書き禁止（生成と git の差分検出で drift 禁止）
- 内容: `global_rules`（compatibility / validity / integrity）+ `artifacts`（artifact_type / owner_team / current_version / schema_versions[content_hash, created] / rules）

## Apicurio Operator CRD（reconcile 経路）
- 採用: Apicurio Registry Operator（Apache-2.0、apicurio.io 公式）
- 使用 CR:
  - `ApicurioRegistry`（Registry instance 定義）
  - `ApicurioRule`（global rule / per-artifact rule）
  - per-artifact / per-version は Registry REST API を Operator が consume する形（CR で全網羅できない場合は薄い additive controller で吸収）
- reconcile 動作:
  1. Argo CD が `apicurio_subjects/` 以下の YAML を apply
  2. Apicurio Operator が CR の差分を検出し、自身の service account（admission allow 対象）で Registry API を呼び subject schema / rule を upsert
  3. drift 検出（Registry 上の state が git と異なる）は Argo CD reconcile + admission block + audit_event
- PII cluster / 業務 cluster の両 Argo CD instance が同 git repo を read-only sync。Registry instance は cluster ごと独立だが state は byte-equal

## 薄い additive controller（必要時のみ）
- Apicurio Operator 標準 CR で表現できない rule 形が判明した場合のみ、薄い additive controller を企画自製（fork ではない、independent controller、Apache-2.0）
- controller の責務:
  - `apicurio_subjects/` 以下の特殊 CR（例: subject group scoped rule、cross-artifact integrity rule）を読む
  - Registry REST API の `/rules` endpoint に upsert
  - reconcile loop は kubebuilder ベース、Registry API client は Apicurio 公式 OpenAPI から自動生成
- lifecycle 規律: 本 controller は 08_OSS ライフサイクル適合仕様 の `v1_inhouse_authoritative` class（`k1s0_apicurio_additive_controller` として inventory 登録）。spec_drift signal source は Apicurio Registry API の OpenAPI 改訂 watch。Apicurio Operator が新 CR で必要 rule を網羅した時点で本 controller は dead inventory として CI fail

## Karapace への demote 経路（L2\* family）
- `apicurio_registry` が migration trigger（license_change / eol_announced 等）で family demote される場合、Karapace primary 化に伴い Registry instance が差替わる
- Karapace は Apicurio の `/rules` API と完全 spec 互換ではないため、demote 時は `apicurio_subjects/` の rule expression を Karapace 互換形式に変換する Software Template を発火（Backstage 配布、Apicurio Operator → Karapace 設定変換）
- git SoT 自体は変更しない。Operator のみ差替え。L2\* governance: gitops_only は維持

## 5 層 defense-in-depth
- 層 A: compile 時。generated codegen 出力（`buf-for-apicurio` plugin、12_client/12 と整合）の入力 hash と `apicurio_rules.lock.yaml` の content_hash が一致することを CI で確認
- 層 B: lint。`apicurio_subjects/*` の全 schema が CUE schema で type 検証 green。本ファイルが他軸（09_data/09, 14、06_スキーマ進化、12_client/12、infra GitOps、security 強制、test 検証規律、08_OSS lifecycle）から参照されることを grep
- 層 C: cross-cluster equality test（test/19 検証規律適合仕様 層 B 整合「Apicurio cross-cluster equality」）。Testcontainers で各 cluster の Apicurio API 応答を取得し git SoT との byte-equal を property test
- 層 D: runtime monitoring。Argo CD reconcile failure / Apicurio Operator reconcile failure を 09 観測可能性 SoR に emit。drift 連続検出は alert + page
- 層 E: admission。`block-direct-apicurio-write` policy が最終 safety net

## ship blocker 解除条件（1.0.0）
- SB-1: `apicurio_subjects/` git repo + `apicurio_rules.lock.yaml` build script + CI 整合検査が green
- SB-2: Apicurio Operator CR が compatibility mode / global rule / per-subject rule を全網羅（網羅できない rule 形が無いことを inventory で確認、または additive controller で補完）
- SB-3: Kyverno admission `block-direct-apicurio-write` が non-Operator service account からの Registry write API call を block（chainsaw / kuttl test）
- SB-4: cross-cluster property test: 同一 subject の compatibility rule / latest version が全 cluster で byte-equal を Testcontainers + 実 Apicurio で property test green
- SB-5: 本仕様が 09_data/09, 14、06_スキーマ進化、08_OSS lifecycle、client 12_コード生成、infra GitOps、security 強制機構、test 検証規律 から参照されること（層 B grep）

## 残リスク
- Apicurio Operator が compatibility rule の全形を CR で表現できない可能性: 薄い additive controller の自製で吸収（fork ではないため lifecycle class は維持、`v1_inhouse_authoritative` inventory に追加）
- git SoT の write 経路の競合: `apicurio_subjects/` への PR レビューを Backstage で SchemaOps role に集約、多 team 同時 PR は branch protection rule で順次 merge
- Apicurio Operator 自体の bug で reconcile が止まる: `infra.gitops.drift.event` 観測 + alert で page、break-glass で manual sync を Backstage 経由 audit 必須
- Karapace demote 時の rule 互換性 gap: Software Template で変換、変換不能な rule は admission block + PR revert を要求（dead spec 殺し原則と整合）

## 関連参照
- [スキーマ進化適合仕様](../01_適合仕様/06_スキーマ進化適合仕様.md)
- [OSS ライフサイクル適合仕様](../01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [client コード生成方針](../../03_概要設計/09_client設計方針/08_コード生成方針.md)
