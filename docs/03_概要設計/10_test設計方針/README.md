---
id: arch.test.test_index
axis: test
phase: architecture
kind: index
status: draft
depends_on:
  - detail.test.verification_conformance
  - detail.test.test_enforcement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# test 設計方針 index

## 一文方針
- 本フォルダは test 軸（13 軸 + meta + security + ops の上位 meta-layer）の概要設計を集約する。test は新規物理機構を持ち込まず、各軸の defense-in-depth 層 A〜E から発信される全 verification claim が必ず coverage matrix（18 axis × 5 verification_class = 90 cell）に bind され、cell ごとの last_green_at が cadence 内に維持されることを assert する meta-layer。

## 至高路線における立ち位置
- test は 16 軸目として 00_軸登録適合仕様 の meta-registry に entry 登録される（cap v1=20 中 18 / 20、残 2 余地）。
- 「test を後で書く」「flaky だから一旦 skip」「coverage は努力目標」を全て禁止し、`coverage_matrix.lock.yaml` の green cell 完全性を 1.0.0 ship blocker とする。
- mutation_score の baseline 引き下げ禁止、quarter monotonic increase only。
- 現実には起こりにくいから property test を書かない / test 実行時間が長いから coverage を間引く判断は採らない。

## test / formal / security / ops との同型直交
- test 軸: 18 axis × 5 verification_class = 90 cell の coverage matrix（実行 artifact）
- formal 軸: 19 axis × 5 proof_class = 95 cell の proof matrix（machine-checkable certificate）
- security 軸: 5⁴ = 625 cell の threat catalog × mitigation pointer
- ops 軸: 5 signal_class × 5 phase = 25 cell の ops_loop catalog × action pointer
- formal と test は同型直交: test の `v1_property_axiom` cell と formal の `v1_temporal_safety_proof` / `v1_program_correctness_proof` cell は cross-cutting に bound（property test corpus は proof obligation の instantiation、proof obligation は property の axiom 形式）

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | verification_class |
|---|---|---|---|
| 01 | [カバレッジ行列方針](01_カバレッジ行列方針.md) | 18 axis × 5 class = 90 cell の coverage_matrix.lock.yaml | -（横断）|
| 02 | [性質ベース検証方針](02_性質ベース検証方針.md) | Hypothesis / fast-check / proptest / jqwik / FsCheck | v1_property_axiom |
| 03 | [契約検証方針](03_契約検証方針.md) | Pact + Pact Broker + Buf-breaking | v1_contract_pair |
| 04 | [シナリオリプレイ方針](04_シナリオリプレイ方針.md) | Playwright + Apache JMeter + chainsaw | v1_scenario_replay |
| 05 | [故障注入検証方針](05_故障注入検証方針.md) | Litmus + Toxiproxy + Chaos Mesh + AFL++ | v1_fault_chaos |
| 06 | [変異検証方針](06_変異検証方針.md) | pitest / mutmut / Stryker / cargo-mutants / go-mutesting | -（test の test）|
| 07 | [退行検証方針](07_退行検証方針.md) | regression corpus + flaky quarantine + golden snapshot | -（横断）|
| 08 | [性能検証方針](08_性能検証方針.md) | Apache JMeter + Locust + k1s0-perf-h3wt | -（性能、独立）|

## 5 verification_class

| class | cadence (day) | active cell 数 (v1 cap) | tool |
|---|---|---|---|
| `v1_type_invariant` | 1 | 17（全 18 軸 - 1）| TypeScript / Rust / Go / Java type check / Buf / jsonschema / CUE |
| `v1_property_axiom` | 7 | 17 | Hypothesis / fast-check / proptest / jqwik / FsCheck |
| `v1_contract_pair` | 3（client15 のみ 14-30） | 17 | Pact / Pact Broker / Buf |
| `v1_scenario_replay` | 14（infra01 / data01 / 11 key / security15 / ops15 は 30）| 17 | Playwright / Apache JMeter / chainsaw |
| `v1_fault_chaos` | 30（infra01=90 / data01=30-180 / security15=90）| 17 | Litmus / Toxiproxy / AFL++ |

合計 18 axis × 5 class = 90 cell（v1 では cap 80 / 80、v2 で 20 軸 × 5 = 100）。

## 5 phase（verification の closure）
- `phase_1_specify` → `phase_2_generate` → `phase_3_execute` → `phase_4_triage` → `phase_5_learn`

## 上位フェーズへの依存
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md): test 軸の責務範囲
- [非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md): 委譲先
- [検証規律要件](../../02_要件定義/03_非機能要件/09_検証規律要件.md): test + formal 軸の要件
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): test 軸採用 OSS

## 下位フェーズへの委譲
- [検証規律適合仕様](../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md): structural spec、build artifact 化
- [test 強制機構](../../04_詳細設計/02_強制機構/09_test強制機構.md): 5 層 defense-in-depth + 25+ Kyverno admission policy
- [test 運用 UI](../../04_詳細設計/04_運用UI開発者体験/06_test運用UI.md)

## 横断軸との bind
- [formal 設計方針](../11_formal設計方針/README.md): 双方向 lock（property axiom ↔ proof obligation）
- [security 設計方針](../07_security設計方針/README.md): security threat 1:1 attacker simulation を v1_fault_chaos blueprint として bind
- [ops 設計方針](../08_ops設計方針/README.md): drill_failure / drill_success の signal 経路、chaos drill の break-glass token

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)
