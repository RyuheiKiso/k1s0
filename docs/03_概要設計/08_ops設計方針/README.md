---
id: arch.ops.ops_index
axis: ops
phase: architecture
kind: index
status: draft
depends_on:
  - detail.ops.ops_loop_conformance
  - detail.ops.ops_enforcement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# ops 設計方針 index

## 一文方針
- 本フォルダは ops 軸（13 軸 + meta + security の上位 meta-layer）の概要設計を集約する。ops は新規物理機構を持ち込まず、5 階層 + security の 14 軸 grid から発信される全 operational signal が必ず運用 loop（detect → respond → resolve → learn → prevent）の closure に到達することを cross-axis に assert する meta-layer。

## 至高路線における立ち位置
- ops は 15 軸目として 00_軸登録適合仕様 の meta-registry に entry 登録される（cap v1=20 中 18 / 20、残 2 余地）。
- 13 軸 + security が「層別 × 機能別 × 脅威別」軸であるのに対し、ops は「signal 別 × phase 別 × 時間軸」軸として直交方向に位置する。
- 「page が増えると on-call が疲弊する」を理由に operational signal coverage を縮小しない。疲弊は fatigue_budget metric で表現、coverage 自体は省略しない。
- 「postmortem の merge を待っていると速度が落ちる」を理由に postmortem ship blocker を緩めない。loop closure 不能の蓄積は長期的に壊滅的。

## test / formal / security との同型直交
- test 軸: 18 axis × 5 verification_class の coverage matrix
- formal 軸: 19 axis × 5 proof_class の proof matrix
- security 軸: 5⁴ = 625 cell の threat catalog × mitigation pointer
- ops 軸: 5 signal_class × 5 phase の ops_loop catalog × action pointer
- 全 cell 物理 enforce の構造的整合体として 1.0.0 ship blocker

## 配下ドキュメント

| 番号 | ドキュメント | 主題 |
|---|---|---|
| 01 | [SLO 駆動運用方針](01_SLO駆動運用方針.md) | error budget burn rate × 5 段階運用 action（observe/notify/page/freeze/rollback）|
| 02 | [アラート方針](02_アラート方針.md) | alert_catalog + 5 severity + page-runbook 1:1 + noise budget |
| 03 | [オンコール方針](03_オンコール方針.md) | 3 階層 escalation + rotation gap zero + fatigue budget |
| 04 | [運用インシデント方針](04_運用インシデント方針.md) | 5 incident class × 6 phase lifecycle + IR command roles |
| 05 | [ポストモーテム方針](05_ポストモーテム方針.md) | blameless + 6 必須 section + action item branch out + ship blocker |
| 06 | [変更管理方針](06_変更管理方針.md) | 5 change class + progressive delivery + auto rollback + freeze gate |
| 07 | [runbook 方針](07_runbook方針.md) | 5 runbook class + Backstage TechDocs + Tekton Pipeline 二段構え |
| 08 | [toil 削減方針](08_toil削減方針.md) | SRE 50% rule + quarterly target monotonic decrease + automation backlog |

## 5 signal_class（ops_loop の primary key）
- `v1_slo_breach`: 13 SLO の error budget burn rate ≥ 6x
- `v1_capacity_breach`: per_tenant_volume / cluster headroom / partition capacity の hard threshold 違反
- `v1_change_induced`: deploy / migration / canary / topology change 直後の SLO 急変
- `v1_dependency_outage`: 上位 / 下位 service / 外部 SaaS / IDP / DNS の unavailability
- `v1_drill_failure`: chaos drill / restore drill / red team drill で想定外 failure mode に到達

## 5 phase（loop closure）
- `phase_1_detect` → `phase_2_triage` → `phase_3_mitigate` → `phase_4_resolve` → `phase_5_postmortem`

## 上位フェーズへの依存
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md): ops の責務範囲
- [非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md): 委譲先
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): ops 軸採用 OSS

## 下位フェーズへの委譲
- [運用ループ適合仕様](../../04_詳細設計/01_適合仕様/17_運用ループ適合仕様.md): structural spec、build artifact 化
- [ops 強制機構](../../04_詳細設計/02_強制機構/07_ops強制機構.md): 5 層 defense-in-depth + 25+ Kyverno admission policy
- [ops 運用 UI](../../04_詳細設計/04_運用UI開発者体験/04_ops運用UI.md)
- [ops_edge_cluster](../../04_詳細設計/03_クロスカッティング適合仕様/10_ops_edge_cluster.md): target cluster outage 時にも escalation を物理発火する独立 cluster

## 横断軸との bind
- [security 設計方針](../07_security設計方針/README.md): security incident（threat detect）の triage 後 operational impact phase は ops loop に entry
- [tier1 SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md): error_budget.lock.yaml が ops の budget_action_bindings.lock.yaml に bind
- [test 設計方針](../10_test設計方針/README.md): chaos drill / runbook drill cadence の双方向 lock
- [formal 設計方針](../11_formal設計方針/README.md): runbook idempotency / progress proof obligation

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)
