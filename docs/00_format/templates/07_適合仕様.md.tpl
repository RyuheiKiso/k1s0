---
id: detail.<axis>.<slug>_conformance
axis: <axis>
phase: detail
kind: conformance_spec
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - <axis>_classes.yaml
  - <axis>_scenarios.yaml
  - <axis>_capabilities.lock.yaml
---

# <axis> <主題> 適合仕様（v1）

## 位置づけ
- <関連方針 / Server 系 / Library 系の意味論的不変量宣言> の規約と、conformance scenario corpus と、<adapter / capability> matrix の三者を、機械可読な単一の真として一箇所に固定する仕様書。
- <関連方針> が「動く層」を定義するのに対し、本仕様は「層が等価に動いていることを CI で機械的に証明する規約」を定義する。
- <CI lint / conformance test runner / Capability Negotiation> のいずれも本仕様が宣言する N つの YAML を共通入力として参照する。

## 設計原則
- <conformance_class は bundle である / dimension override は禁止する / dead spec を CI で殺す 等の規律 3 点>
- 単一 OSS 深耕。<該当 OSS と機能カテゴリ>。
- 物理 enforcement。文章 only の合意は禁止。
- 段階的 release 禁止。1.0.0 時点で全 cell が verified。

## v1 class セット（N class）

| class | <dim 1> | <dim 2> | <dim 3> | <dim 4> | <dim 5> |
|---|---|---|---|---|---|
| <v1_class_a> | ... | ... | ... | ... | ... |
| <v1_class_b> | ... | ... | ... | ... | ... |

### 各 class の不変条件と典型用途

#### <v1_class_a>
- <不変条件>
- <典型用途>
- <delivery / consistency / SLA 性質>

#### <v1_class_b>
- <不変条件>
- <典型用途>

## dimension の根拠
- <dim 1> = <値> の存在意義: <根拠>
- <dim X 不採用の理由>
- direction × adapter の構造的不両立: <該当時のみ>

## 単一の真の N ファイル構成

### `<axis>/conformance/classes.yaml`
- class bundle 定義。conformance_class 名 → 各 dimension の純粋関数テーブル。
- スケッチ:
  ```yaml
  classes:
    v1_class_a:
      <dim_1>: <value>
      <dim_2>: <value>
  ```

### `<axis>/conformance/scenarios.yaml`
- scenario corpus 定義。assertion id → 期待 invariants。
- スケッチ:
  ```yaml
  scenarios:
    sc_<assertion_id>:
      class: v1_class_a
      assert: <invariant>
  ```

### `<axis>/conformance/capabilities.lock.yaml`
- adapter capability matrix。adapter 名 → supports class 配列。
- generator output として build artifact 化、手書き禁止。
- スケッチ:
  ```yaml
  adapters:
    <adapter_name>:
      supports: [v1_class_a, v1_class_b]
  ```

## 整合 check（CI 強制）
- 整合 1: classes.yaml 全 class が proto option enum と 1:1 対応
- 整合 2: scenarios.yaml 全 assertion が classes.yaml の class を参照
- 整合 3: capabilities.lock.yaml 全 adapter の supports が classes.yaml の class subset
- 整合 4: unreachable class 検出（どの adapter にも supports されない class は CI fail）
- 整合 5: dead spec 検出（参照されない class / assertion / adapter は CI fail）

## 5 層 defense-in-depth
- 層 A: compile（schema / type check）
- 層 B: lint（Buf custom lint / Conftest）
- 層 C: integration test（conformance test runner）
- 層 D: runtime（capability negotiation / admission webhook）
- 層 E: 物理（cosign signed conformance result / Object Lock）
- 層 F: 数学的（proof certificate; formal 関連時のみ）

## 強制機構との bind
- 本仕様は [<axis> 強制機構](../02_強制機構/<axis>強制機構.md) の以下経路で enforce:
    - <enforce 経路 1>
    - <enforce 経路 2>
- release_gate.lock.yaml cell: `<axis>_<slug>_conformance` → 1.0.0 ship blocker

## 関連参照
- <関連 spec / 関連方針 への link>
